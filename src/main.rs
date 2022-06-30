use device_query::{DeviceQuery, DeviceState, Keycode};
use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use nobela_parser2::{parse_flat, server};

fn main() {
    let input = r#"
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
"Well, that's about all we have for now..."
	"#;

    let stmts = parse_flat(input).unwrap_or_else(|e| panic!("{}", e));
    let mut events = server::Server::new(&stmts, 0);

    let mut e = events.next();

    let mut just_pressed = Vec::new();
    let mut pressed = Vec::new();

    while let Some(ref event) = e {
        match event {
            server::Event::Dialogue {
                speaker,
                text,
                choices,
            } => {
                show_dialogue(speaker, text);
                if !choices.is_empty() {
                    let choice_index = choose(choices);
                    events.choose(choice_index)
                } else {
                    enum Nav {
                        Next,
                        Back,
                    }
                    let mut nav: Option<Nav> = None;

                    println!("<- Back		Next ->");

                    while nav.is_none() {
                        let device_state = DeviceState::new();
                        let keys: Vec<Keycode> = device_state.get_keys();

                        just_pressed.clear();

                        let mut remove_indexes = Vec::new();

                        for (i, key) in pressed.iter().enumerate() {
                            if !keys.contains(key) {
                                remove_indexes.push(i)
                            }
                        }

                        for i in remove_indexes {
                            pressed.remove(i);
                        }

                        for key in keys {
                            if !pressed.contains(&key) {
                                just_pressed.push(key);
                                pressed.push(key);
                            }
                        }

                        nav = if (just_pressed.contains(&Keycode::Right)
                            || just_pressed.contains(&Keycode::Enter))
                            && !just_pressed.contains(&Keycode::Left)
                        {
                            Some(Nav::Next)
                        } else if just_pressed.contains(&Keycode::Left)
                            && !(just_pressed.contains(&Keycode::Right)
                                || just_pressed.contains(&Keycode::Enter))
                        {
                            Some(Nav::Back)
                        } else {
                            None
                        };
                    }
                }
            }
            server::Event::Ignore => (),
        }
        e = events.next();
    }

    clear()
}

fn show_dialogue(speaker: &Option<String>, text: &String) {
    clear();
    let output = if let Some(speaker) = speaker {
        format!("{speaker}: {text}")
    } else {
        format!(": {text}")
    };
    println!("{output}\n");
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

fn clear() {
    print!("{}[2J", 27 as char);
}
