

mod support;

#[derive(Default)]
struct Participant {
    name: String
}

impl Participant {
    fn share(&self, state: &State) -> f32 {
        state.total / state.participants.len() as f32
    }
}

#[derive(Default)]
struct Receipt {
    label: String,
    total: f32,
    excluded: Vec<Receipt>
}

#[derive(Default)]
struct State {
    participants: Vec<Participant>,
    receipts: Vec<Receipt>,
    total: f32,

    tmp_name: String,
    tmp_label: String,
    tmp_total: f32,

    r_tmp_label: String,
    r_tmp_total: f32
}

fn main() {
    let mut state = State::default();
    state.total = 69.99;

    let system = support::init(file!());

    system.main_loop(move |_, ui| {
        // Here we create a window with a specific size, and force it to always have a vertical scrollbar visible
        ui.window("Big complex window")
            .size([800.0, 500.0], imgui::Condition::FirstUseEver)
            .position([0., 0.], imgui::Condition::Always)
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .menu_bar(false)
            .title_bar(false)
            .always_vertical_scrollbar(true)
            .bring_to_front_on_focus(false)
            .build(|| {

                // Note you can create windows inside other windows, however, they both appear as separate windows.
                // For example, somewhere deep inside a complex window, we can quickly create a widget to display a
                // variable, like a graphical "debug print"


                // Show the more Rust'y iterator
                ui.window("Participants")
                .size([250.0, 380.0], imgui::Condition::FirstUseEver)
                .position([10., 10.], imgui::Condition::Always)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .build(|| {
                    let clipper = imgui::ListClipper::new(state.participants.len() as i32)
                        .items_height(ui.current_font_size())
                        .begin(ui);

                    for row_num in clipper.iter() {
                        if let Some(participant) = &state.participants.get(row_num as usize) {
                            ui.tree_node_config(format!("{}: {}##{row_num}", participant.name, participant.share(&state)))
                            .build(|| {
                                if ui.button("Remove") {
                                    state.participants.remove(row_num as usize);
                                }
                            });
                        }
                    }
                });

                ui.window("Add participant")
                .size([250.0, 90.0], imgui::Condition::FirstUseEver)
                .position([10., 400.], imgui::Condition::Always)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .scroll_bar(false)
                .scrollable(false)
                .build(|| {
                    ui.text("Participant's name");
                    ui.input_text("##add_participant", &mut state.tmp_name)
                    .hint("e.g. Matti Heikkinen")
                    .enter_returns_true(true)
                    .build();

                    {
                        let _danger_token = ui.begin_disabled(!(state.tmp_name.len() > 0));
                        if ui.button("Add") {
                            state.participants.push(Participant { name: state.tmp_name.clone() });
                            state.tmp_name = String::new();
                        }
                    }
                });

                ui.window("Receipts")
                .size([300.0, 250.0], imgui::Condition::FirstUseEver)
                .position([480., 10.], imgui::Condition::Always)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .build(|| {
                    let clipper = imgui::ListClipper::new(state.receipts.len() as i32)
                    .items_height(ui.current_font_size())
                    .begin(ui);

                    for row_num in clipper.iter() {
                        if let Some(receipt) = &state.receipts.get_mut(row_num as usize) {
                            ui.tree_node_config(format!("{}: {}##{row_num}", receipt.label, receipt.total))
                            .build(|| {

                                ui.text("Popout individual item");
                                ui.text("Item's label");
                                ui.input_text("##item_label", &mut state.r_tmp_label)
                                    .hint("e.g. Toothbrush")
                                    .enter_returns_true(true)
                                    .build();
            
                                ui.text("Item's cost");
                                ui.input_float("##item_total", &mut state.r_tmp_total)
                                .build();
                                
                                if ui.button("Exclude") {
                                    println!("E");
                                }

                                if ui.button("Remove") {
                                    state.receipts.remove(row_num as usize);
                                }
                            });
                        }
                    }
                });

                ui.window("Add Receipt")
                .size([300.0, 150.0], imgui::Condition::FirstUseEver)
                .position([480., 270.], imgui::Condition::Always)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .build(|| {
                    ui.text("Receipt's label");
                    ui.input_text("##receipt_label", &mut state.tmp_label)
                        .hint("e.g. Shopping trip")
                        .enter_returns_true(true)
                        .build();

                    ui.text("Receipt's total");
                    ui.input_float("##receipt_total", &mut state.tmp_total)
                    .build();

                    {
                        let _danger_token = ui.begin_disabled(!(state.tmp_label.len() > 0) || state.tmp_total <= 0.);
                        if ui.button("Add") {
                            println!("Add");
                            state.receipts.push(Receipt { label: state.tmp_label.clone(), total: state.tmp_total.clone(), excluded: vec![] });
                            state.tmp_label = String::new();
                            state.tmp_total = 0.;
                        }
                    }

                });
            });
    });
}