use proc_macro::TokenStream;
use syn::{spanned::Spanned, Error as SynError, Result as SynResult};

#[derive(Debug)]
pub enum AttributeData {
    Default,
    Full {
        output_type: String,
        post_processor: String,
    },
}

pub fn parse_attributes(attr: TokenStream) -> SynResult<AttributeData> {
    if attr.is_empty() {
        return Ok(AttributeData::Default);
    }

    let tokens = attr.to_string();
    let Some((output_type, post_processor)) = tokens.rsplit_once(',') else {
        return Err(SynError::new(
            tokens.span(),
            "expects an output type and a post-processor function, seperated by a comma",
        ));
    };

    Ok(AttributeData::Full {
        output_type: output_type.trim().to_string(),
        post_processor: post_processor.trim().to_string(),
    })
}
