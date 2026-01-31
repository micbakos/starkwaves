use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Attribute, Meta};

/// Derive macro for parsing Starknet events.
///
/// # Example
///
/// ```rust
/// #[derive(StarknetEvent)]
/// pub enum GameEvent {
///     PlayersAssembled {
///         #[key] game_id: Felt,
///         player_a: Felt,
///         player_b: Felt,
///     },
///     GameStarted {
///         #[key] game_id: Felt,
///         attacker: Felt,
///         defender: Felt,
///     },
/// }
/// ```
///
/// This generates a `TryFrom<starknet::core::types::Event>` implementation
/// that parses events based on their selector (derived from variant name).
///
/// Fields marked with `#[key]` are read from `event.keys` (after the selector).
/// Other fields are read from `event.data`.
#[proc_macro_derive(StarknetEvent, attributes(key, event))]
pub fn derive_starknet_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("StarknetEvent can only be derived for enums"),
    };

    let match_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let event_name = get_event_name(variant_name, &variant.attrs);

        let fields = match &variant.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("StarknetEvent variants must have named fields"),
        };

        let mut key_fields = Vec::new();
        let mut data_fields = Vec::new();

        for field in fields {
            let field_name = field.ident.as_ref().unwrap();
            let is_key = field.attrs.iter().any(|attr| attr.path().is_ident("key"));

            if is_key {
                key_fields.push(field_name);
            } else {
                data_fields.push(field_name);
            }
        }

        let key_count = key_fields.len();
        let data_count = data_fields.len();

        let key_extractions = key_fields.iter().enumerate().map(|(i, field_name)| {
            let idx = i + 1; // Skip selector at index 0
            quote! {
                #field_name: *event.keys.get(#idx)
                    .ok_or_else(|| EventParseError::MissingKey {
                        event: #event_name.to_string(),
                        index: #idx
                    })?,
            }
        });

        let data_extractions = data_fields.iter().enumerate().map(|(i, field_name)| {
            quote! {
                #field_name: *event.data.get(#i)
                    .ok_or_else(|| EventParseError::MissingData {
                        event: #event_name.to_string(),
                        index: #i
                    })?,
            }
        });

        quote! {
            selector if selector == &starknet::core::utils::get_selector_from_name(#event_name)
                .expect("Invalid event name") =>
            {
                if event.keys.len() < #key_count + 1 {
                    return Err(EventParseError::InsufficientKeys {
                        event: #event_name.to_string(),
                        expected: #key_count + 1,
                        got: event.keys.len(),
                    });
                }
                if event.data.len() < #data_count {
                    return Err(EventParseError::InsufficientData {
                        event: #event_name.to_string(),
                        expected: #data_count,
                        got: event.data.len(),
                    });
                }

                Ok(#name::#variant_name {
                    #(#key_extractions)*
                    #(#data_extractions)*
                })
            }
        }
    }).collect::<Vec<_>>();

    let expanded = quote! {
        #[derive(Debug, Clone)]
        pub enum EventParseError {
            MissingSelector,
            UnknownEvent { selector: starknet::core::types::Felt },
            MissingKey { event: String, index: usize },
            MissingData { event: String, index: usize },
            InsufficientKeys { event: String, expected: usize, got: usize },
            InsufficientData { event: String, expected: usize, got: usize },
        }

        impl std::fmt::Display for EventParseError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    EventParseError::MissingSelector => write!(f, "Event has no selector"),
                    EventParseError::UnknownEvent { selector } => write!(f, "Unknown event selector: {}", selector),
                    EventParseError::MissingKey { event, index } => write!(f, "Missing key at index {} for event {}", index, event),
                    EventParseError::MissingData { event, index } => write!(f, "Missing data at index {} for event {}", index, event),
                    EventParseError::InsufficientKeys { event, expected, got } => write!(f, "Event {} expected {} keys, got {}", event, expected, got),
                    EventParseError::InsufficientData { event, expected, got } => write!(f, "Event {} expected {} data fields, got {}", event, expected, got),
                }
            }
        }

        impl std::error::Error for EventParseError {}

        impl TryFrom<starknet::core::types::Event> for #name {
            type Error = EventParseError;

            fn try_from(event: starknet::core::types::Event) -> Result<Self, Self::Error> {
                let selector = event.keys.get(0)
                    .ok_or(EventParseError::MissingSelector)?;

                match selector {
                    #(#match_arms)*
                    _ => Err(EventParseError::UnknownEvent { selector: *selector }),
                }
            }
        }

        impl TryFrom<&starknet::core::types::Event> for #name {
            type Error = EventParseError;

            fn try_from(event: &starknet::core::types::Event) -> Result<Self, Self::Error> {
                let selector = event.keys.get(0)
                    .ok_or(EventParseError::MissingSelector)?;

                match selector {
                    #(#match_arms)*
                    _ => Err(EventParseError::UnknownEvent { selector: *selector }),
                }
            }
        }

        impl TryFrom<starknet::core::types::EmittedEvent> for #name {
            type Error = EventParseError;

            fn try_from(event: starknet::core::types::EmittedEvent) -> Result<Self, Self::Error> {
                let selector = event.keys.get(0)
                    .ok_or(EventParseError::MissingSelector)?;

                match selector {
                    #(#match_arms)*
                    _ => Err(EventParseError::UnknownEvent { selector: *selector }),
                }
            }
        }

        impl TryFrom<&starknet::core::types::EmittedEvent> for #name {
            type Error = EventParseError;

            fn try_from(event: &starknet::core::types::EmittedEvent) -> Result<Self, Self::Error> {
                let selector = event.keys.get(0)
                    .ok_or(EventParseError::MissingSelector)?;

                match selector {
                    #(#match_arms)*
                    _ => Err(EventParseError::UnknownEvent { selector: *selector }),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Get the event name from the variant.
/// If #[event(name = "...")] is specified, use that.
/// Otherwise, use the variant name as-is.
fn get_event_name(variant_name: &Ident, attrs: &[Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("event") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens = meta_list.tokens.to_string();
                // Parse name = "EventName"
                if let Some(name) = tokens.strip_prefix("name = \"") {
                    if let Some(name) = name.strip_suffix("\"") {
                        return name.to_string();
                    }
                }
            }
        }
    }

    // Default: use variant name as-is
    variant_name.to_string()
}
