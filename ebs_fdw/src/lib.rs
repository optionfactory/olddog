#![allow(unused)]

use std::collections::HashMap;
use supabase_wrappers::interface::{ForeignDataWrapper, Limit, Qual, Row, Sort};
use supabase_wrappers::wrappers_fdw;
use aws_sdk_ec2::{Client, Region};
use aws_sdk_ec2::error::DescribeVolumesError;
use aws_sdk_ec2::model::*;
use aws_sdk_ec2::types::SdkError;
use futures_core::Stream;
use futures_util::stream::StreamExt;
use supabase_wrappers::prelude::*;
use tokio::runtime::Runtime;

// creates the metadata required to make our cdylib a valid postgres extension
pgrx::pg_module_magic!();

// marks the struct as a foreign data wrapper, to export the necessary functions
#[wrappers_fdw(
version = "0.1.0",
author = "OptionFactory",
)]
pub struct EbsFdw {
    runtime: Runtime,
    client: Client,
    columns: Vec<Column>,
    items: Option<Box<dyn Stream<Item=Result<Volume, SdkError<DescribeVolumesError>>> + Unpin>>,
}

// FDW implementation
impl ForeignDataWrapper for EbsFdw {

    // Called once for every query, and should initialize what is required
    fn new(options: &HashMap<String, String>) -> Self {
        let region = Region::new(options.get("region").expect("Missing option 'region'").to_string());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");
        let client = runtime.block_on(async {
            let config = aws_config::ConfigLoader::default().region(region).load().await;
            Client::new(&config)
        });
        Self {
            runtime,
            client,
            columns: Vec::default(),
            items: None,
        }
    }

    // Called by postgres at scan start
    // Parameters detail what is requested, including where clauses, sorts, limit
    fn begin_scan(&mut self, quals: &[Qual], columns: &[Column], _sorts: &[Sort], _limit: &Option<Limit>, _options: &HashMap<String, String>) {

        // uncomment to enable predicate pushdown
        // let filters = quals_to_filters(quals);
        let filters = None;

        // start an async request to AWS API
        let items = self.client.describe_volumes()
            .set_filters(filters)
            .into_paginator()
            .items()
            .send();
        // save the iterator and projection list for later
        self.items = Some(Box::new(items));
        self.columns = columns.to_vec();
    }

    // Called repeatedly by postgres to fetch rows
    fn iter_scan(&mut self, row: &mut Row) -> Option<()> {

        // Retrieve the next item from the flattening paginator
        let item = self.runtime.block_on(async {
            self.items.as_mut().expect("no items")
                .next().await
                .map(|e| e.expect("Failed to retrieve items"))
        });

        // If we have a next item from EBS, populate postgres row with values
        if let Some(vol) = item {
            row.clear();
            for column in &self.columns {
                match column.name.as_str() {
                    "id" => row.push("id", vol.volume_id().map(|id| Cell::String(id.to_string()))),
                    "name" => row.push("name", vol.tags().and_then(name_tag).map(Cell::String)),
                    "type" => row.push("type", vol.volume_type().map(|t| Cell::String(t.as_str().to_string()))),
                    "size" => row.push("size", vol.size().map(Cell::I32)),
                    "encrypted" => row.push("encrypted", vol.encrypted().map(Cell::Bool)),
                    c => row.push(c, None),
                }
            }
            Some(())
        } else {
            None
        }
    }

    fn end_scan(&mut self) {
        // Nothing to do
    }
}

/// Extract name tag
fn name_tag(tags: &[Tag]) -> Option<String> {
    tags.iter()
        .find(|t| t.key() == Some("Name"))
        .and_then(|t| t.value())
        .map(str::to_string)
}

/// Utility function to do predicate pushdown, converting quals to AWS API filters
fn quals_to_filters(quals: &[Qual]) -> Option<Vec<Filter>> {
    let filters = quals.iter()
        .filter_map(|qual|
            match qual {
                Qual { field, operator, value: Value::Cell(value), use_or: false, .. } if operator == "=" => {
                    Some(Filter::builder().name(field).values(value.to_string()).build())
                },
                _ => None,
            })
        .collect::<Vec<_>>();
    if filters.is_empty() {
        return None
    }
    Some(filters)
}
