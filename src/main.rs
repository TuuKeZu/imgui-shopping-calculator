use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

mod support;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Participant {
    id: Uuid,
    name: String,
}

impl Participant {
    fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
        }
    }

    fn share(&self, state: &State) -> f32 {
        state.share_map().get(&self.name).iter().fold(0., |s, map| {
            s + map.iter().fold(0., |s, (_, total)| s + total)
        })
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Receipt {
    label: String,
    total: f32,
    id: Uuid,
    exclusion: bool,
}

impl Receipt {
    fn new(label: String, total: f32, exlusion: bool) -> Self {
        Self {
            label,
            total,
            id: Uuid::new_v4(),
            exclusion: exlusion,
        }
    }

    fn total(&self, state: &State) -> f32 {
        if self.exclusion {
            return self.total;
        }

        let a = state
            .exclusions
            .iter()
            .filter(|(id, _)| self.id == **id)
            .fold(0., |s, a| s + a.1.total);
        f32::max(self.total - a, 0.)
    }
}

#[derive(Default)]
struct State {
    participants: Vec<Participant>,
    share_map: HashMap<Uuid, Vec<Uuid>>,
    receipts: Vec<Receipt>,
    exclusions: HashMap<Uuid, Receipt>,

    tmp_name: String,
    tmp_label: String,
    tmp_total: f32,

    r_tmp_label: String,
    r_tmp_total: f32,

    tmp_auto_add: bool,
}

impl State {
    fn share_map(&self) -> HashMap<String, HashMap<String, f32>> {
        let mut map: HashMap<String, HashMap<String, f32>> = HashMap::new();

        for receipt in self.receipts.iter() {
            let participants: Vec<&Uuid> = self
                .share_map
                .iter()
                .filter(|(_, list)| list.contains(&receipt.id))
                .map(|(id, _)| id)
                .collect();
            let share = receipt.total(self) / participants.len() as f32;

            for id in participants.iter() {
                let name = self
                    .participants
                    .iter()
                    .find(|p| p.id == **id)
                    .unwrap()
                    .name
                    .clone();
                if map.contains_key(&name) {
                    map.get_mut(&name)
                        .unwrap()
                        .insert(receipt.label.clone(), share);
                } else {
                    map.insert(name, HashMap::from([(receipt.label.clone(), share)]));
                }
            }
        }

        map
    }

    fn total(&self) -> f32 {
        self.share_map().iter().fold(0., |s, (_, map)| {
            s + map.iter().fold(0., |s, (_, total)| s + total)
        })
    }

    fn export_csv(&self) -> String {
        let mut s = String::new();
        let map = self.share_map();

        s += "Name,Total,Receipt\n";

        for (name, inner) in map.iter() {
            let total = inner.iter().fold(0., |s, (_, total)| s + total);
            s += &format!("{name},{total:.2}€,\n");
            for (label, total) in inner.iter() {
                s += &format!(",{total:.2}€,{label}\n");
            }
        }

        s += &format!("Total,{:.2}€,", self.total());
        println!("{s}");
        s
    }

    fn export_txt(&self) -> String {
        let mut s = String::new();
        let map = self.share_map();

        fn f(s: &str, l: usize) -> String {
            if s.len() >= l {
                s.to_string()
            } else {
                let n = l - s.len();
                s.to_string() + &" ".repeat(n)
            }
        }

        let c1 = usize::max(map.keys().map(|x| x.len()).max().unwrap(), 5);
        let c2 = usize::max(
            map.values()
                .map(|m| m.values().map(|x| format!("{x:.2}€").len()).max().unwrap())
                .max()
                .unwrap(),
            5,
        );
        let c3 = usize::max(
            map.values()
                .map(|m| m.keys().max().unwrap().len())
                .max()
                .unwrap(),
            7,
        );

        s += &format!(
            "{}   {}   {}\n",
            f("Name", c1),
            f("Total", c2 - 2),
            f("Receipt", c3)
        );
        s += &format!("{}\n", "-".repeat(9 + c1 + c2 + c3));

        for (name, inner) in map.iter() {
            let total = inner.iter().fold(0., |s, (_, total)| s + total);

            s += &format!(
                "{}   {}   {}\n",
                f(&name, c1),
                f(&format!("{total:.2}€"), c2),
                f("", c3)
            );

            for (label, total) in inner.iter() {
                s += &format!(
                    "{} > {}   {}\n",
                    f("", c1),
                    f(&format!("{total:.2}€"), c2),
                    f(&label, c3)
                );
            }
        }

        s += &format!("{}\n", "-".repeat(9 + c1 + c2 + c3));
        s += &format!(
            "{}   {}   {}",
            f("Total", c1),
            f(&format!("{:.2}€", self.total()), c2),
            f("", c3)
        );

        println!("{s}");
        s
    }
}

fn main() {
    let mut state = State::default();

    // Initialize
    let r = Receipt::new("Sello".to_string(), 69.99, false);
    let id = r.id;
    state.receipts.push(r);
    let e = Receipt::new("item".to_string(), 2.99, true);
    state.exclusions.insert(id, e.clone());
    state.receipts.push(e);

    let system = support::init(file!());

    system.main_loop(move |_, ui| {
        // Here we create a window with a specific size, and force it to always have a vertical scrollbar visible
        ui.window("##main_frame")
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
                                let id = participant.id;
                                ui.tree_node_config(format!(
                                    "{}: {:.2}##{row_num}",
                                    participant.name,
                                    participant.share(&state)
                                ))
                                .build(|| {
                                    ui.tree_node_config(format!("Manage receipts##{row_num}"))
                                        .build(|| {
                                            if state.receipts.is_empty() {
                                                ui.text_disabled("No receipts to share");
                                            } else {
                                                ui.text("Currently partaking in: ");
                                                ui.text_disabled("------------------");
                                                for receipt in state.receipts.iter() {
                                                    let selected =
                                                        state.share_map[&id].contains(&receipt.id);

                                                    if ui
                                                        .selectable_config(format!(
                                                            "{} ({})##{row_num}",
                                                            receipt.label,
                                                            if selected {
                                                                "partake"
                                                            } else {
                                                                "ignore"
                                                            }
                                                        ))
                                                        .allow_double_click(false)
                                                        .selected(selected)
                                                        .build()
                                                    {
                                                        if selected {
                                                            state
                                                                .share_map
                                                                .get_mut(&id)
                                                                .unwrap()
                                                                .retain(|i| i != &receipt.id)
                                                        } else {
                                                            state
                                                                .share_map
                                                                .get_mut(&id)
                                                                .unwrap()
                                                                .push(receipt.id);
                                                        }
                                                    }
                                                }
                                                ui.text_disabled("------------------");
                                            }
                                        });

                                    if ui.button("Remove") {
                                        let p = state.participants.remove(row_num as usize);
                                        state.share_map.remove(&p.id);
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
                                let p = Participant::new(state.tmp_name.clone());
                                let id = p.id;
                                state.participants.push(p);
                                state
                                    .share_map
                                    .insert(id, state.receipts.iter().map(|r| r.id).collect());
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
                            if let Some(receipt) = &state.receipts.get(row_num as usize) {
                                let id = receipt.id;
                                let total = receipt.total(&state);
                                let exclusion = receipt.exclusion;
                                let exclusions: Vec<(String, f32)> = state
                                    .exclusions
                                    .iter()
                                    .filter(|(id, _)| receipt.id == **id)
                                    .map(|(_, e)| (e.label.clone(), e.total))
                                    .collect();

                                ui.tree_node_config(format!(
                                    "{}: {:.2}##{row_num}",
                                    receipt.label, total
                                ))
                                .build(|| {
                                    if !exclusion {
                                        ui.tree_node_config(format!(
                                            "Item exclusions ({})",
                                            exclusions.len()
                                        ))
                                        .build(|| {
                                            for (label, total) in exclusions {
                                                ui.text_colored(
                                                    [255., 0., 0., 255.],
                                                    format!("- {:.2} ({})", total, label),
                                                );
                                            }
                                        });

                                        ui.tree_node_config("Manage Exclusions").build(|| {
                                            ui.text_disabled("----------------------");
                                            ui.text_disabled("Item's name");
                                            ui.input_text("##item_label", &mut state.r_tmp_label)
                                                .hint("e.g. Toothbrush")
                                                .enter_returns_true(true)
                                                .build();

                                            ui.text_disabled("Item's cost");
                                            ui.input_float("##item_total", &mut state.r_tmp_total)
                                                .build();

                                            if ui.button("Exclude") {
                                                let receipt = Receipt::new(
                                                    state.r_tmp_label.clone(),
                                                    state.r_tmp_total.clone(),
                                                    true,
                                                );
                                                state.exclusions.insert(id, receipt.clone());
                                                state.receipts.push(receipt.clone());

                                                if state.tmp_auto_add {
                                                    state
                                                        .share_map
                                                        .iter_mut()
                                                        .for_each(|(_, l)| l.push(receipt.id));
                                                }

                                                state.r_tmp_label = String::new();
                                                state.r_tmp_total = 0.;
                                            }
                                            ui.text_disabled("----------------------");
                                        });
                                    } else {
                                        ui.text_disabled("(This individual item is excluded)")
                                    }

                                    if ui.button("Remove") {
                                        let re = state.receipts.remove(row_num as usize);

                                        if re.exclusion {
                                            let p = state
                                                .exclusions
                                                .iter()
                                                .find(|(_, r)| r.id == re.id)
                                                .map(|r| r.0.clone());

                                            if p.is_none() {
                                                return;
                                            }

                                            state.exclusions.remove(&p.unwrap());
                                        }
                                    }
                                });
                            }
                        }
                    });

                ui.window("Add Receipt")
                    .size([300.0, 180.0], imgui::Condition::FirstUseEver)
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

                        ui.tree_node_config("Options").build(|| {
                            if ui
                                .selectable_config(format!(
                                    "auto-share ({})",
                                    if state.tmp_auto_add {
                                        "enabled"
                                    } else {
                                        "disabled"
                                    }
                                ))
                                .selected(state.tmp_auto_add)
                                .build()
                            {
                                state.tmp_auto_add = !state.tmp_auto_add;
                            }
                        });

                        {
                            let _danger_token = ui.begin_disabled(
                                !(state.tmp_label.len() > 0) || state.tmp_total <= 0.,
                            );
                            if ui.button("Add") {
                                let receipt = Receipt::new(
                                    state.tmp_label.clone(),
                                    state.tmp_total.clone(),
                                    false,
                                );
                                state.receipts.push(receipt.clone());

                                if state.tmp_auto_add {
                                    state
                                        .share_map
                                        .iter_mut()
                                        .for_each(|(_, l)| l.push(receipt.id));
                                }

                                state.tmp_label = String::new();
                                state.tmp_total = 0.;
                            }
                        }
                    });

                ui.window("Exporting options")
                    .size([200.0, 100.0], imgui::Condition::FirstUseEver)
                    .position([270., 10.], imgui::Condition::Always)
                    .resizable(false)
                    .movable(false)
                    .collapsible(false)
                    .build(|| {
                        ui.text("Current total");
                        ui.text_disabled(format!("{:.2}", state.total()));

                        if ui.button("Export in .csv") {
                            state.export_csv();
                        }

                        if ui.button("Export in .txt") {
                            state.export_txt();
                        }
                    })
            });
    });
}
