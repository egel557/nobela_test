use std::collections::HashMap;

use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use evalexpr::context_map;
use nobela_parser2::{parse_flat, server};

fn main() {
    let input = r#"
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
	"#;

    let timeline = parse_flat(input).unwrap_or_else(|e| panic!("{}", e));
    let context = context_map! {
        "foo" => "bar"
    }
    .unwrap();
    let mut events = server::Server::new(server::Config {
        timelines: HashMap::new(),
        timeline_stack: vec![&timeline],
        index_stack: vec![0],
        context: &context,
    });

    let mut e = events.next();

    while let Some(ref event) = e {
        match event {
            server::Event::Dialogue {
                speaker,
                text,
                choices,
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
