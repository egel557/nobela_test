use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use evalexpr::{context_map, Context, ContextWithMutableVariables};
use nobela::{parser, server};

fn main() {
	let characters = parser::characters_from_json("characters.json").unwrap();
    let parser = parser::Parser::new(characters);
	let timelines = parser.parse_dir("timelines").unwrap_or_else(|e| panic!("{}", e));
    let mut context = context_map! {
        "foo" => "bar",
        "msg" => "This is a message"
    }
    .unwrap();

    let mut events = server::Server::new(timelines, context.to_owned());

	events.start("start", 0);

    let mut e = events.next();

    while let Some(ref event) = e {
        match event {
            server::Event::Dialogue {
                speaker,
                text,
                choices,
                ..
            } => {
                show_dialogue(speaker, text);

                let mut cs = Vec::new();

                for choice in choices {
                    if !choice.1 {
                        cs.push(choice.0.to_owned())
                    }
                }

                if !cs.is_empty() {
                    let choice_index = choose(&cs);
                    events.choose(choice_index)
                } else {
                    nav();
                }
            }
            server::Event::Ignore => (),
            server::Event::Set {
                variable_name,
                new_value,
            } => {
                context
                    .get_value(variable_name)
                    .unwrap_or_else(|| panic!("Unknown variable '{variable_name}'"));
                context
                    .set_value(variable_name.to_owned(), new_value.to_owned())
                    .unwrap();
                events.set_context(context.to_owned());
            }
        }
        e = events.next();
    }
}

fn show_dialogue(speaker: &Option<String>, text: &String) {
	let output = if let Some(speaker) = speaker {
        format!("{speaker}: {text}")
    } else {
        format!(": {text}")
    };
    println!("{output}\n");
}

fn nav() {
    Select::with_theme(&ColorfulTheme::default())
        .items(&["Next"])
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap();
}

fn choose(choices: &[String]) -> usize {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(choices)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    if let Some(selection) = selection {
        selection
    } else {
        0
    }
}
