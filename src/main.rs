use cursive::Cursive;
use cursive::event::{Event, Key};
use cursive::menu::MenuTree;
use cursive::view::ScrollStrategy;
use cursive::views::{Button, Dialog, DummyView, EditView, LinearLayout, ListView, NamedView, ResizedView, ScrollView, SelectView, TextArea, TextView};
use cursive::traits::{Boxable, Nameable};

const _DEBUG: bool = true;
const USERNAME: &str = "guest";

struct LiveData {
	alias: String,
	alias_list: Vec<String>,
}

impl LiveData {
	fn new() -> LiveData {
		LiveData {
			alias: USERNAME.to_string(),
			alias_list: vec![USERNAME.to_string()],
		}
	}
}

fn main() {
	let mut siv = cursive::default();
	// siv.toggle_debug_console();

	siv.menubar()
		.add_subtree("Help", MenuTree::new()
			.leaf("General", |s| {
				s.add_layer(Dialog::info("General help coming soon"))
			})
			.leaf("About", |s| {
				s.add_layer(Dialog::info(&format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))));
			}));
	
	siv.set_autohide_menu(false);
	
	siv.add_global_callback(Key::Esc, |s| s.select_menubar());

	siv.add_global_callback(Event::Shift(Key::Enter), |s | { // Shift+Enter to send
		send_message(s);
	});

	let mut chat = ScrollView::new(ListView::new()
			.with_name("chat"))
		.with_name("chat_scroll_view");
	
	let mut chat_mut = chat.get_mut();
	chat_mut.set_scroll_strategy(ScrollStrategy::StickToBottom);
	chat_mut.set_on_scroll_change(|s: &mut Cursive, _| {reset_scroll_strategy(s)});
	
	let chat = chat.full_height();
	
	let chat_input = TextArea::new()
		.with_name("chat_input")	
		.fixed_size((90, 5));
	
	let buttons = LinearLayout::vertical()
		.child(Button::new("Send", |s| {
			send_message(s);
			// Set focus on chat
			s.focus_name("chat_input").unwrap();
		}))
		.child(Button::new("Aliases", |s| {
			let alias_select = s.find_name::<SelectView<String>>("alias_select");

			match alias_select {
				None => {},
				Some(mut a) => {
					a.clear();

					for alias in &get_user_data(s).alias_list {
						a.add_item_str(alias);
					}
				}
			};

			s.add_layer(LinearLayout::horizontal()
				.child(SelectView::<String>::new()
					.on_submit(|s, item: &str| {
						get_user_data(s).alias = item.to_string();
					})
					.with_name("alias_select")
					.fixed_size((10, 5)))
				.child(LinearLayout::vertical()
					.child(Button::new("Select", select_alias))
					.child(Button::new("Add", add_alias))
					.child(Button::new("Remove", remove_alias))
					.child(Button::new("Back", |s| {
						s.pop_layer();
					}))));
		}))
		.child(Button::new("Quit", |s| {
			s.add_layer(Dialog::around(TextView::new("Do you really want to quit?"))
				.dismiss_button("No")
				.button("Yes", |s| {
					s.quit();
				}));
		}));
	
	let input = LinearLayout::horizontal()
		.child(chat_input)
		.child(DummyView)
		.child(buttons);
	
	let chat_application = LinearLayout::vertical()
		.child(chat)
		.child(DummyView)
		.child(input);
	
	siv.add_layer(chat_application);

	let live_user_data = LiveData::new();

	siv.set_user_data(live_user_data);

	siv.run();
}

fn send_message(s: &mut Cursive) {
	let mut message_find = s.find_name::<TextArea>("chat_input").unwrap(); // TextArea / EditView
	let message = message_find.get_content();

	send(s, message);
	message_find.set_content("");
}

fn send(s: &mut Cursive, msg: &str) {
	if msg.as_bytes().iter().all(u8::is_ascii_whitespace) { // If all characters are whitespace
		return;
	}

	let mut chat_find = s.find_name::<ListView>("chat").unwrap();
	let message_number = chat_find.len() + 1;
	let alias = &get_user_data(s).alias;

	chat_find.add_child(&format!("{} {}", message_number.to_string(), alias), TextView::new(msg));
}

// Resets the scroll strategy to stick to the bottom if it is manually scrolled down there
fn reset_scroll_strategy(s: &mut Cursive) {
	let mut chat_scroll_view = s.find_name
		::<ResizedView<NamedView<ScrollView<NamedView<ListView>>>>>("chat_scroll_view")
			.unwrap()
			.get_inner_mut()
			.get_mut();
	
	if chat_scroll_view.is_at_bottom() {
		chat_scroll_view.set_scroll_strategy(ScrollStrategy::StickToBottom);
	}
}

fn _debug(s: &mut Cursive, msg: &str) {
	if _DEBUG {
		send(s, &format!("debug: {}", msg));
	}
}

fn _dialog_debug(s: &mut Cursive, msg: &str) {
	if _DEBUG {
		s.add_layer(Dialog::info(msg));
	}
}

fn add_alias(s: &mut Cursive) {
	fn ok(s: &mut Cursive, name: &str) {
		_debug(s, "1");
		s.call_on_name("alias_select", |view: &mut SelectView<String>| {
			view.add_item_str(name);
		});
		_debug(s, "2");

		let alias_list = &mut get_user_data(s).alias_list;
		alias_list.insert(alias_list.len() + 1, name.to_string());
		_debug(s, "3");

		s.pop_layer();
		_debug(s, "4");
	}

	_debug(s, "5");
	s.add_layer(Dialog::around(EditView::new()
			.on_submit(ok)
			.with_name("alias_dialog")
			.fixed_width(10))
		.title("Enter a new alias")
		.button("OK", |s| {
			let name = s.call_on_name("alias_dialog", |view: &mut EditView| {
				view.get_content()
			}).unwrap();
			ok(s, &name);
		})
		.button("Cancel", |s| {
			s.pop_layer();
		}));
	_debug(s, "6");
}

fn remove_alias(s: &mut Cursive) {
	let mut alias_select = s.find_name::<SelectView<String>>("alias_select").unwrap();

	match alias_select.selected_id() {
		None => s.add_layer(Dialog::info("No alias to remove.")),
		Some(focus) => {
			alias_select.remove_item(focus);
		}
	}
}

fn select_alias(s: &mut Cursive) {
	let alias_select = s.find_name::<SelectView<String>>("alias_select").unwrap();

	match alias_select.selected_id() {
		None => s.add_layer(Dialog::info("No alias to select.")),
		Some(focus) => {
			let alias = alias_select.get_item(focus).unwrap().0;
			get_user_data(s).alias = alias.to_string();
		}
	}
}

fn get_user_data(s: &mut Cursive) -> &mut LiveData {
	s.user_data::<LiveData>().unwrap()
}
