use std::{env, fs};

extern crate proc_macro;
use proc_macro::*;

extern crate core;

fn quote_section(section: Option<&str>, properties: Vec<TokenTree>) -> proc_macro2::TokenStream {
	let section = match section {
		Some(name) => quote::quote!(Option::Some(#name)),
		None => quote::quote!(Option::<&'static str>::None),
	};
	let properties = proc_macro2::TokenStream::from(properties.into_iter().collect::<TokenStream>());
	quote::quote! {
		(
			#section,
			&[#properties]
		)
	}
}

#[proc_macro]
pub fn import(tokens: TokenStream) -> TokenStream {
	let lit_str = syn::parse_macro_input!(tokens as syn::LitStr);

	let path = lit_str.value();

	let path = if path.starts_with("/") {
		env::current_dir().unwrap().join(&path[1..]).to_str().unwrap().to_owned()
	}
	else { panic!("paths cannot be relative, they must start with / which is the project root") };

	let ini_data = fs::read_to_string(&path).expect(&path);

	let mut sections = Vec::new();
	let mut sections_count = 1usize;
	let mut section = None;
	let mut properties = Vec::new();

	for (line, item) in ini_core::Parser::new(&ini_data).enumerate() {
		match item {
			ini_core::Item::Error(_) | ini_core::Item::Action(_)=> {
				panic!("syntax error at line {}", line);
			},
			ini_core::Item::Section(name) => {
				sections.push(quote_section(section, properties));
				sections_count += 1;
				properties = Vec::new();
				section = Some(name);
			},
			ini_core::Item::Property(key, value) => {
				let group = Group::new(Delimiter::Parenthesis, vec![
					TokenTree::Literal(Literal::string(key)),
					TokenTree::Punct(Punct::new(',', Spacing::Alone)),
					TokenTree::Literal(Literal::string(value)),
				].into_iter().collect());
				properties.push(TokenTree::Group(group));
				properties.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
			},
			ini_core::Item::Comment(_) | ini_core::Item::Blank => {
			},
		}
	}
	sections.push(quote_section(section, properties));

	(quote::quote! {
		{
			// Rerun the macro if input file changes
			let _ = ::core::include_str!(#path);
			// Annotate the INI with types by putting it through a const
			use ::core::option::Option;
			const INI: [(Option<&str>, &[(&str, &str)]); #sections_count] = [#(#sections,)*];
			&INI
		}
	}).into()
}
