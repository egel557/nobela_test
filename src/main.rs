use std::collections::HashMap;

use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use evalexpr::{context_map, ContextWithMutableVariables};
use nobela_parser::{NobelaParser, server::{self}};

fn main() {
	let parser = NobelaParser::new(vec![]);
    let timeline1 = parser.parse(
        r#"
"This is the first timeline."
jump "timeline_2"
"Back to first timeline."
"#,
    )
    .unwrap_or_else(|e| panic!("{}", e));
    let timeline2 = parser.parse(
        r#"
foo = "Hello World"
if foo == "Hello World":
	"Assign successful!"
"..."
-- "This should show" if true
-- "This shouldn't" if false
if true:
	"This should show."
	if true:
		"This too!"
if false:
	"This shouldn't show."
"Hey, there! This is a demo for Nobela!"
"This is just a regular dialogue. Cool, huh?"
"This one has choices!"
-- "Cool!"
	"Glad you thought so!"
	"We actually took a different route when you made that choice!"
	"So this dialogue is nested inside of that choice you made!"
	"Within this nested dialogue, you can also have choices!"
	-- "Whoa!"
	-- "Also Whoa!"
-- "Meh..."
	"..."
	"Someone's hard to impress..."
	"We took a different route when you made that choice..."
	"This dialogue is now nested inside of that choice..."
	"Does that impress you?"
	-- "Yes!"
	-- "Sure, I guess..."
		"..."
	-- "Still meh."
		"..."
"We also have a friend here with us!"
"Say hi, Friend!"
"Friend" "I'm not your friend."
		"#,
    )
    .unwrap_or_else(|e| panic!("{}", e));
    let mut context = context_map! {
        "foo" => "bar"
    }
    .unwrap();
	
	let mut config = server::Config {
		timelines: HashMap::new(),
		timeline_stack: vec![&timeline1],
		index_stack: vec![0],
		context: context.to_owned(),
	};

	config.timelines.insert("timeline_1".to_owned(), &timeline1);
	config.timelines.insert("timeline_2".to_owned(), &timeline2);

    let mut events = server::Server::new(config);

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
            server::Event::Set { variable_name, new_value } => {
				context.set_value(variable_name.to_owned(), new_value.to_owned()).unwrap();
				events.set_context(context.to_owned());
			},
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
