use std::{env, process};

use dialoguer::{Select, theme::ColorfulTheme, console::Term, Input};
use nobela_parser2::{parse_flat, parse_nested, FlatStmt, NestedStmt};


fn main() {
	let args: Vec<String> = env::args().collect();
	let mode = args.get(1).unwrap_or_else(|| {
        eprintln!("No argument for mode provided. Please enter either 'flat' or 'nested'");
        process::exit(1);
    });
	
	let input =
	r#"
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

	if args.len() > 1 {
		match mode as &str {
			"flat" => {
				let events = parse_nested(input).unwrap_or_else(|e| panic!("{}", e));
				execute_nested(&events);
			},
			"nested" => {
				let events = parse_flat(input).unwrap_or_else(|e| panic!("{}", e));
				execute_flat(&events);
			},
			_ => {
				eprintln!("Invalid mode provided. Please enter either 'flat' or 'nested'");
        		process::exit(1);
			}
		}
	}

	clear()
	
}

fn execute_choice(choice: &NestedStmt) {
    if let NestedStmt::Choice { children, .. } = choice {
        for event in children {
            match event {
                NestedStmt::Dialogue { .. } => execute_dialogue(event),
                NestedStmt::Choice { .. } => (),
            }
        }
    }
}

fn execute_dialogue(dialogue: &NestedStmt) {
    if let NestedStmt::Dialogue {
        choices, ..
    } = dialogue
    {
		show_dialogue_nested(dialogue);
		
		if choices.is_empty() {
			wait_enter()
		} else {
			let choice_index = choose_nested(choices);
			execute_choice(&choices[choice_index])
		}
    }
}
fn execute_nested(events: &Vec<NestedStmt>) {
    for event in events {
        match event {
            NestedStmt::Dialogue { .. } => execute_dialogue(event),
            NestedStmt::Choice { .. } => (),
        }
    }
}

fn execute_flat(events: &Vec<FlatStmt>) {
    let mut index = 0;

    while index < events.len() {
		let event = &events[index];
        match event {
            FlatStmt::Dialogue { .. } => {
                let mut next_index = index + 1;
                let mut choices = Vec::new();
                let mut choice_indexes = Vec::new();
                let mut nested_count = 0;

                loop {
					let next_event = &events[next_index];
                    match next_event {
                        FlatStmt::EndDialogue => {
                            if nested_count > 0 {
                                nested_count -= 1
                            } else {
                                break;
                            }
                        }
                        FlatStmt::Dialogue { .. } => nested_count += 1,
                        FlatStmt::Choice { .. } => {
                            if nested_count > 0 {
                                nested_count += 1
                            } else {
                                choices.push(next_event);
                                choice_indexes.push(next_index.to_owned())
                            }
                        }
                        FlatStmt::EndChoice => {
                            if nested_count > 0 {
                                nested_count -= 1
                            }
                        } 
                    }
                    next_index += 1;
                }

                show_dialogue_flat(&event);

				index = if choices.is_empty() {
					wait_enter();
					index + 1
				} else {
					choice_indexes[choose_flat(&choices)]
				}

                // index = if !choice_indexes.is_empty() {
                //     choice_indexes[0]
                // } else {
                //     index + 1
                // }; // Change depending on user choice
            }
            FlatStmt::Choice { .. } => {
                index += 1;
            }
            FlatStmt::EndDialogue => {
                index += 1;
            }
            FlatStmt::EndChoice => {
                let mut next_index = index + 1;
                let mut nested_count = 0;
				let mut next_event = &events[next_index];

                if matches!(next_event, FlatStmt::Choice { .. }) {
                    loop {
                        match next_event {
                            FlatStmt::Dialogue { .. } => nested_count += 1,
                            FlatStmt::EndDialogue => {
                                if nested_count > 0 {
                                    nested_count -= 1
                                } else {
                                    break;
                                }
                            }
                            FlatStmt::Choice { .. } => nested_count += 1,
                            FlatStmt::EndChoice => nested_count -= 1,
                        }
                        next_index += 1;
						next_event = &events[next_index];
                    }
                }
                index = next_index;
            }
        }
    }
}


fn wait_enter() {
	Input::<String>::new()
	.with_prompt("(Continue...)")
	.allow_empty(true)
	.report(false)
    .interact_text().unwrap();
}

fn choose_nested(choices: &Vec<NestedStmt>) -> usize {
	let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&choices.into_iter().map(|c| to_string_nested(c)).collect::<Vec<String>>())
        .default(0)
        .interact_on_opt(&Term::stderr()).unwrap();

	if let Some(selection) = selection {
		selection
	} else {
		0
	}
}

fn show_dialogue_nested(event: &NestedStmt) {
	clear();
	println!("{}\n", to_string_nested(event));
}

fn to_string_nested(event: &NestedStmt) -> String {
	match event {
		NestedStmt::Dialogue { speaker, text, .. } => {
			if let Some(speaker) = speaker {
				format!("{speaker}: {text}")
			} else {
				format!(": {text}")
			}
		},
		NestedStmt::Choice { text, .. } => text.to_owned(),
	}
}

fn choose_flat(choices: &Vec<&FlatStmt>) -> usize {
	let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&choices.into_iter().map(|c| to_string_flat(c)).collect::<Vec<String>>())
        .default(0)
        .interact_on_opt(&Term::stderr()).unwrap();

	if let Some(selection) = selection {
		selection
	} else {
		0
	}
}

fn show_dialogue_flat(event: &FlatStmt) {
	clear();
	println!("{}\n", to_string_flat(event));
}

fn to_string_flat(event: &FlatStmt) -> String {
	match event {
		FlatStmt::Dialogue { speaker, text, .. } => {
			if let Some(speaker) = speaker {
				format!("{speaker}: {text}")
			} else {
				format!(": {text}")
			}
		},
		FlatStmt::Choice { text, .. } => text.to_owned(),
		_ => String::new()
	}
}

fn clear() {
	print!("{}[2J", 27 as char);
}