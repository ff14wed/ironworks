use std::{collections::HashMap, iter::once};

use ironworks_schema_core::{Node, ReferenceCondition, ReferenceTarget};
use serde_json::Value;

use crate::{
	error::{Error, Result},
	lcs::longest_common_subsequence,
};

/// See also:
/// - [SheetDefinition.cs#L157](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/Definition/SheetDefinition.cs#L157)
/// - [PositionedDataDefinition.cs#L71](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/Definition/PositionedDataDefinition.cs#L71)
pub fn parse_sheet_definition(value: &Value) -> Result<Node> {
	let nodes = iter_value_field(value, "definitions")
		.map(|definition| {
			// PositionedDataDefinition inlined as it's only used in one location, and makes setting up the struct fields simpler
			let index = definition.get("index").and_then(Value::as_u64).unwrap_or(0);
			let (node, name) = parse_data_definition(definition)?;

			Ok((
				name.unwrap_or_else(|| format!("Unnamed{}", index)),
				(index.try_into().unwrap(), node),
			))
		})
		.collect::<Result<HashMap<_, _>>>()?;

	Ok(Node::Struct(nodes))
}

/// See also:
/// - [IDataDefinition.cs#L34](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/Definition/IDataDefinition.cs#L34)
fn parse_data_definition(value: &Value) -> Result<(Node, Option<String>)> {
	match value.get("type").and_then(Value::as_str) {
		None => parse_single_data_definition(value),
		Some("group") => parse_group_data_definition(value),
		Some("repeat") => parse_repeat_data_definition(value),
		Some(unknown) => Err(Error::Schema(format!("Unknown data type {}", unknown))),
	}
}

/// See also:
/// - [SingleDataDefinition.cs#L66](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/Definition/SingleDataDefinition.cs#L66)
fn parse_single_data_definition(value: &Value) -> Result<(Node, Option<String>)> {
	let name = value.get("name").and_then(Value::as_str).map(String::from);

	let converter = match value.get("converter") {
		Some(object) => object,
		None => return Ok((Node::Scalar, name)),
	};

	// TODO: There's also a "quad" type with a converter but I've got no idea how it's instantiated.
	let node = match converter.get("type").and_then(Value::as_str) {
		Some("color") => parse_color_converter(converter),
		Some("generic") => parse_generic_reference_converter(converter),
		Some("icon") => parse_icon_converter(converter),
		Some("multiref") => parse_multi_reference_converter(converter),
		Some("link") => parse_sheet_link_converter(converter),
		Some("tomestone") => parse_tomestone_or_item_reference_converter(converter),
		Some("complexlink") => parse_complex_link_converter(converter),
		unknown => Err(Error::Schema(format!(
			"Unknown converter type {}",
			unknown.unwrap_or("(none)")
		))),
	};

	Ok((node?, name))
}

/// See also:
/// - [GroupDataDefinition.cs#L125](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/Definition/GroupDataDefinition.cs#L125)
fn parse_group_data_definition(value: &Value) -> Result<(Node, Option<String>)> {
	let nodes = iter_value_field(value, "members")
		.scan(0u32, |size, member| {
			Some(parse_data_definition(member).map(|(node, name)| {
				let current_size = *size;
				*size += node.size();

				(
					name.unwrap_or_else(|| format!("Unnamed{}", current_size)),
					(current_size, node),
				)
			}))
		})
		.collect::<Result<HashMap<_, _>>>()?;

	// StC doesn't give groups a name for some reason -
	// try to derive a name from the LCS of its keys
	let name = nodes
		.keys()
		.cloned()
		.reduce(|a, b| longest_common_subsequence(&a, &b));

	Ok((Node::Struct(nodes), name))
}

/// See also:
/// - [RepeatDataDefinition.cs#L85](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/Definition/RepeatDataDefinition.cs#L85)
fn parse_repeat_data_definition(value: &Value) -> Result<(Node, Option<String>)> {
	// TODO: These... as well as all the other errors, really... have no way to pinpoint _where_ the error occured. Look into it.
	let definition = value
		.get("definition")
		.ok_or_else(|| Error::Schema("Repeat missing definition".to_string()))?;

	let count = value
		.get("count")
		.and_then(Value::as_u64)
		.ok_or_else(|| Error::Schema("Repeat missing count".to_string()))?;

	let (node, name) = parse_data_definition(definition)?;

	Ok((Node::Array(count.try_into().unwrap(), Box::new(node)), name))
}

/// See also:
/// - [ColorConverter.cs#L46](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/ValueConverters/ColorConverter.cs#L46)
fn parse_color_converter(_value: &Value) -> Result<Node> {
	// TODO: ?
	Ok(Node::Scalar)
}

/// See also:
/// - [GenericReferenceConverter.cs#L33](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/ValueConverters/GenericReferenceConverter.cs#L33)
fn parse_generic_reference_converter(_value: &Value) -> Result<Node> {
	// TODO: ?
	Ok(Node::Scalar)
}

/// See also:
/// - [IconConverter.cs#L33](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/ValueConverters/IconConverter.cs#L33)
fn parse_icon_converter(_value: &Value) -> Result<Node> {
	// TODO: ?
	Ok(Node::Scalar)
}

/// See also:
/// - [MultiReferenceConverter.cs#L50](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/ValueConverters/MultiReferenceConverter.cs#L50)
fn parse_multi_reference_converter(value: &Value) -> Result<Node> {
	let targets = iter_value_field(value, "targets")
		.filter_map(Value::as_str)
		.map(|target| ReferenceTarget {
			sheet: target.to_string(),
			selector: None,
			condition: None,
		})
		.collect::<Vec<_>>();

	Ok(Node::Reference(targets))
}

/// See also:
/// - [SheetLinkConverter.cs#L40](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/ValueConverters/SheetLinkConverter.cs#L40)
fn parse_sheet_link_converter(value: &Value) -> Result<Node> {
	let target = value
		.get("target")
		.and_then(Value::as_str)
		.ok_or_else(|| Error::Schema("Link missing target".to_string()))?;

	Ok(Node::Reference(vec![ReferenceTarget {
		sheet: target.to_string(),
		selector: None,
		condition: None,
	}]))
}

/// See also:
/// - [TomestoneOrItemReferenceConverter.cs#L54](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/ValueConverters/TomestoneOrItemReferenceConverter.cs#L54)
fn parse_tomestone_or_item_reference_converter(_value: &Value) -> Result<Node> {
	// TODO: ?
	Ok(Node::Scalar)
}

/// See also:
/// - [ComplexLinkConverter.cs#L143](https://github.com/xivapi/SaintCoinach/blob/800eab3e9dd4a2abc625f53ce84dad24c8579920/SaintCoinach/Ex/Relational/ValueConverters/ComplexLinkConverter.cs#L143)
fn parse_complex_link_converter(value: &Value) -> Result<Node> {
	// TODO: Look into projection

	let mut targets = Vec::<ReferenceTarget>::new();
	for link in iter_value_field(value, "links") {
		let condition = link.get("when").map(parse_when_clause).transpose()?;
		let selector = link.get("key").and_then(Value::as_str).map(str::to_string);

		let sheets = once(link.get("sheet").and_then(Value::as_str))
			.chain(iter_value_field(link, "sheets").map(Value::as_str))
			.flatten()
			.map(|sheet| ReferenceTarget {
				sheet: sheet.to_string(),
				selector: selector.clone(),
				condition: condition.clone(),
			});

		targets.extend(sheets);
	}

	Ok(Node::Reference(targets))
}

fn parse_when_clause(value: &Value) -> Result<ReferenceCondition> {
	let key = value
		.get("key")
		.and_then(Value::as_str)
		.ok_or_else(|| Error::Schema("When clause missing key".to_string()))?;

	let condition_value = value
		.get("value")
		.and_then(Value::as_u64)
		.ok_or_else(|| Error::Schema("When clause missing value".to_string()))?;

	Ok(ReferenceCondition {
		selector: key.to_string(),
		value: condition_value.try_into().unwrap(),
	})
}

/// Iterate over a field within a value, if it exists. If the field does not
/// exist, behaves as an empty iterator.
#[inline]
fn iter_value_field<'a>(value: &'a Value, field: &str) -> impl Iterator<Item = &'a Value> {
	value
		.get(field)
		.and_then(Value::as_array)
		.into_iter()
		.flatten()
}
