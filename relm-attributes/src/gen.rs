/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use std::collections::HashMap;
use std::sync::Mutex;

use quote::Tokens;
use syn::Ident;

use parser::Widget;

lazy_static! {
    static ref NAMES_INDEX: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
}

pub fn gen(name: &Ident, widget: Widget, root_widget: &mut Option<Ident>) -> Tokens {
    let mut widget_names = vec![];
    let widget = gen_widget(&widget, None, &mut widget_names, root_widget);
    let widget_names1 = &widget_names;
    let widget_names2 = &widget_names;
    quote! {
        #widget

        #name {
            #(#widget_names1: #widget_names2),*
        }
    }
}

fn gen_widget(widget: &Widget, parent: Option<&Ident>, widget_names: &mut Vec<Ident>, root_widget: &mut Option<Ident>) -> Tokens {
    let widget_name = &Ident::new(gen_widget_name(&widget.name));
    let struct_name = Ident::new(widget.name.as_ref());
    widget_names.push(widget_name.clone());

    let mut params = Tokens::new();
    for param in &widget.init_parameters {
        params.append(param);
        params.append(",");
    }

    let mut events = vec![];
    for (name, event) in &widget.events {
        let return_value =
            if widget.name == "gtk::Window" && name == "delete_event" {
                quote! {
                    ::gtk::Inhibit(false)
                }
            }
            else {
                quote! {
                    ()
                }
            };
        let event_ident = Ident::new(format!("connect_{}", name));
        let event_params: Vec<_> = event.params.iter().map(|ident| Ident::new(ident.as_ref())).collect();
        let event_name = Ident::new(event.name.as_ref());
        events.push(quote! {
            connect!(relm, #widget_name, #event_ident(#(#event_params),*) {
                (Some(#event_name), #return_value)
            });
        });
    }

    let children: Vec<_> = widget.children.iter()
        .map(|child| gen_widget(child, Some(widget_name), widget_names, root_widget)).collect();

    let add_child_or_show_all =
        if let Some(name) = parent {
            quote! {
                #name.add(&#widget_name);
            }
        }
        else {
            *root_widget = Some(widget_name.clone());
            quote! {
                #widget_name.show_all();
            }
        };

    let mut properties = vec![];
    for (key, value) in &widget.properties {
        let property_func = Ident::new(format!("set_{}", key));
        let mut val = Tokens::new();
        val.append(value);
        properties.push(quote! {
            #widget_name.#property_func(#val);
        });
    }

    quote! {
        let #widget_name = #struct_name::new(#params);
        #(#properties)*
        #(#events)*
        #(#children)*
        #add_child_or_show_all
    }
}

fn gen_widget_name(name: &str) -> String {
    let name =
        if let Some(index) = name.rfind(':') {
            name[index + 1 ..].to_lowercase()
        }
        else {
            name.to_lowercase()
        };
    let mut hashmap = NAMES_INDEX.lock().unwrap();
    let index = hashmap.entry(name.clone()).or_insert(0);
    *index += 1;
    format!("{}{}", name, index)
}