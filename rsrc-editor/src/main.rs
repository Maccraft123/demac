use binrw::BinRead;
use clap::Parser;
use eframe::egui;
use egui::{RichText, Rect};
use macfmt::i18n::RegionCode;
use macfmt::macbinary::{MacBinary2, is_macbinary2};
use macfmt::rsrc::{Resource, ResourceType};
use macfmt::rsrc::types::{
    DevelopmentStage, ItemType, KeyboardShortcut, MarkingCharacter, MenuItem, MenuItemConfig,
    SizeFlags, Type,
};
use macfmt::single::AppleFile;
use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::PathBuf;
use strum::IntoEnumIterator;

mod util;
use util::icon_editor;

#[derive(Parser)]
struct Args {
    file: PathBuf,
}

fn main() -> eframe::Result {
    env_logger::init();
    let args = Args::parse();
    let mut file = File::open(&args.file).unwrap();
    let mut magic = [0u8; 128];
    file.read_exact(&mut magic).unwrap();
    file.rewind().unwrap();

    let res = if is_macbinary2(&magic) {
        let tmp = MacBinary2::read(&mut file).unwrap();
        let mut cursor = Cursor::new(tmp.resource_fork());
        Resource::read(&mut cursor).unwrap()
    } else {
        match magic[0..4] {
            [0x00, 0x05, 0x16, 0x07] | [0x00, 0x05, 0x16, 0x0] => {
                let tmp = AppleFile::read(&mut file).unwrap();
                let mut cursor = Cursor::new(tmp.resource_fork().unwrap());
                Resource::read(&mut cursor).unwrap()
            }
            _ => Resource::read(&mut file).unwrap(),
        }
    };

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "Charcoal".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("charcoal.ttf"))),
    );

    fonts.families.insert(egui::FontFamily::Name("Charcoal".into()), vec!["Charcoal".to_owned()]);

    let luts: Vec<(String, Vec<(u16, image::Rgb<u16>)>)> = res
        .iter()
        .filter_map(|entry| {
            /*if let Type::ColorLut(clut) = entry.data() {
                let name = entry.name().unwrap_or("<unnamed>");
                let mut vec = Vec::new();
                for lut in clut.entries() {
                    vec.push((lut.pixel(), lut.rgb()))
                }
                Some((name.to_string(), vec))
            } else {*/
                None
            //}
        })
        .collect();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(MyApp {
                res,
                cur_res: None,
                cur_ty: None,
                scene_rect: Rect::ZERO,
                luts,
                lut: None,
            }))
        }),
    )
}

struct MyApp {
    res: Vec<(ResourceType, Vec<Resource>)>,
    cur_ty: Option<(usize, ResourceType)>,
    cur_res: Option<usize>,
    scene_rect: Rect,
    luts: Vec<(String, Vec<(u16, image::Rgb<u16>)>)>,
    lut: Option<usize>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            res: Vec::new(),
            cur_res: None,
            cur_ty: None,
            scene_rect: Rect::ZERO,
            luts: Vec::new(),
            lut: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("Resource list").show(ctx, |ui| {
            let ty_text = if let Some(ty) = &self.cur_ty {
                format!("{}", ty.1.inner())
            } else {
                String::new()
            };
            egui::ComboBox::from_id_salt("Resource Type")
                .width(ui.available_width())
                .selected_text(RichText::new(ty_text).monospace())
                .show_ui(ui, |ui| {
                    let old = self.cur_ty.clone();
                    for (i, (ty, _)) in self.res.iter().enumerate() {
                        let text = RichText::new(format!("{}", ty.inner())).monospace();
                        ui.selectable_value(&mut self.cur_ty, Some((i, ty.clone())), text);
                    }
                    if self.cur_ty != old {
                        self.cur_res = None;
                    }
                });

            if let Some((ty, _)) = &self.cur_ty {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                    egui::Grid::new("Type selector").show(ui, |ui| {
                        ui.label("ID");
                        ui.label("Name");
                        ui.end_row();

                        for (i, res) in self.res[*ty].1.iter().enumerate() {
                            ui.selectable_value(&mut self.cur_res, Some(i), format!("{}", res.id()));
                            if let Some(name) = res.name().as_ref() {
                                ui.label(format!("{:?}", name));
                            }
                            ui.end_row();
                        }
                    });
                });
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some((ty_idx, _)) = self.cur_ty && let Some(idx) = self.cur_res {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        let res = &mut self.res[ty_idx].1[idx];
                        egui::ComboBox::from_label("Heap")
                            .selected_text(if res.system_heap {
                                "System"
                            } else {
                                "Application"
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut res.system_heap, true, "System");
                                ui.selectable_value(&mut res.system_heap, false, "Application");
                            });
                        ui.checkbox(&mut res.purgeable, "Purgeable");
                        ui.checkbox(&mut res.locked, "Locked");
                        ui.checkbox(&mut res.protected, "Protected");
                        ui.checkbox(&mut res.preload, "Preload");
                        ui.checkbox(&mut res.compressed, "Compressed");
                        match res.data_mut() {
                            Type::String(s) => {
                                ui.text_edit_multiline(s.as_mut());
                            },
                            Type::KeyboardName(s) => {
                                ui.text_edit_singleline(s.as_mut());
                            },
                            Type::Bundle(bundle) => {
                                ui.label(format!("Signature: {}", bundle.sig()));
                                for ty in bundle.types() {
                                    ui.label(format!("Resource type {}:", ty.type_name()));
                                    for map in ty.res_map() {
                                        
                                    ui.label(format!("Local ID: {}", map.local));
                                        ui.label(format!("Actual ID: {}", map.actual));
                                    }
                                }
                            }
                            Type::FileReference(fref) => {
                                ui.label(format!("File type: {}", fref.ty()));
                                ui.label(format!("Icon ID: {}", fref.icon_id()));
                                ui.label(format!("Filename: {}", fref.filename()));
                            }
                            Type::StringList(list) => {
                                for s in list.list_mut() {
                                    ui.text_edit_multiline(s.as_mut());
                                }
                            }
                            Type::ColorLut(lut) => {
                                for entry in lut.entries_mut() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Pixel: {}", entry.pixel()));
                                        let mut colors: [f32; 3] = entry.rgb_f32();
                                        ui.color_edit_button_rgb(&mut colors);
                                        entry.set_rgb_f32(colors);
                                    });
                                }
                            }
                            Type::FinderIcon(icon) => {
                                ui.add(icon_editor(icon.bw_mut(), &mut self.scene_rect));
                            }
                            Type::Cursor(crsr) => {
                                ui.add(icon_editor(crsr.img_mut(), &mut self.scene_rect));
                            }
                            Type::Pattern(img) => {
                                ui.add(icon_editor(img, &mut self.scene_rect));
                            }
                            Type::Icon(img) => {
                                ui.add(icon_editor(img, &mut self.scene_rect));
                            }
                            Type::SmallIcons(img) => {
                                ui.add(icon_editor(img, &mut self.scene_rect));
                            }
                            Type::LargeColorIcon4(img) => {
                                let mut cursor = std::io::Cursor::new(Vec::new());
                                img.write_to(&mut cursor).unwrap();
                                let uri = format!("bytes://{:?}-{}.bmp", res.ty(), res.id());
                                ui.add(egui::Image::from_bytes(uri, cursor.into_inner()));
                            }
                            Type::LargeColorIcon8(img) => {
                                let mut cursor = std::io::Cursor::new(Vec::new());
                                let lut_text = self
                                    .lut
                                    .map(|idx| self.luts[idx].0.as_str())
                                    .unwrap_or("Default");

                                egui::ComboBox::from_label("Color LUT")
                                    .selected_text(lut_text)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.lut, None, "Default");
                                        for (i, lut) in self.luts.iter().enumerate() {
                                            ui.selectable_value(&mut self.lut, Some(i), &lut.0);
                                        }
                                    });

                                let lut = self.lut.map(|v| self.luts[v].1.as_slice());
                                img.write_to(&mut cursor, lut).unwrap();
                                let uri =
                                    format!("bytes://{:?}-{}-{}.bmp", res.ty(), res.id(), lut_text);
                                ui.add(egui::Image::from_bytes(uri, cursor.into_inner()));
                            }
                            Type::ItemList(ditl) => {
                                let offset_x = ui.cursor().min.x;
                                let offset_y = ui.cursor().min.y;
                                egui::Frame::new()
                                    .stroke(egui::Stroke {
                                        width: 1.0_f32,
                                        color: egui::Color32::RED,
                                    })
                                    .show(ui, |ui| {
                                        ui.style_mut().override_font_id = Some(egui::FontId {
                                            size: 12.0,
                                            family: egui::FontFamily::Name("Charcoal".into()),
                                        });

                                        for item in ditl.items_mut() {
                                            let place = egui::Rect {
                                                min: egui::Pos2 {
                                                    x: item.rect().top_left.x as f32 + offset_x,
                                                    y: item.rect().top_left.y as f32 + offset_y,
                                                },
                                                max: egui::Pos2 {
                                                    x: item.rect().bottom_right.x as f32 + offset_x,
                                                    y: item.rect().bottom_right.y as f32 + offset_y,
                                                },
                                            };

                                            //println!("item {:#?}", item);
                                            match item.data_mut() {
                                                ItemType::Button { text } => {
                                                    if text.len() == 0 {
                                                        ui.put(place, egui::Separator::default());
                                                    } else {
                                                        ui.put(
                                                            place,
                                                            egui::Button::new(text.as_str()),
                                                        );
                                                    }
                                                }
                                                ItemType::Checkbox { text } => {
                                                    ui.put(
                                                        place,
                                                        egui::Checkbox::new(
                                                            &mut false,
                                                            text.as_str(),
                                                        ),
                                                    );
                                                }
                                                ItemType::StaticText { text } => {
                                                    let mut job = egui::text::LayoutJob::default();
                                                    job.justify = false;
                                                    job.halign = egui::Align::Min; // why is this not working
                                                    job.append(text.as_str(), 0.0, egui::TextFormat {
                                                        font_id: egui::FontId::new(12.0, egui::FontFamily::Name("Charcoal".into())),
                                                        color: egui::Color32::WHITE,
                                                        ..Default::default()
                                                    });
                                                    ui.put(
                                                        place,
                                                        egui::Label::new(job),
                                                    );
                                                }
                                                ItemType::EditableText { text } => {
                                                    ui.put(
                                                        place,
                                                        egui::TextEdit::singleline(text.as_mut()),
                                                    );
                                                }
                                                ItemType::AppDefined { .. } => {
                                                    ui.allocate_rect(place, egui::Sense::empty());
                                                    ui.painter().rect_stroke(
                                                        place,
                                                        1.0,
                                                        egui::Stroke {
                                                            width: 1.0,
                                                            color: egui::Color32::GRAY,
                                                        },
                                                        egui::StrokeKind::Inside,
                                                    );
                                                }
                                                _ => (),
                                            }
                                        }
                                    });
                            }
                            Type::Menu(menu) => {
                                let mut enableds = [false; 32];
                                for offset in 0..32 {
                                    enableds[offset] = menu.state() & (1 << offset) != 0;
                                }
                                ui.checkbox(&mut enableds[0], "Menu enabled");

                                ui.horizontal(|ui| {
                                    ui.label("Title:");
                                    ui.text_edit_singleline(menu.title_mut());
                                });
                                
                                ui.label("Items:");
                                let mut to_remove = None;
                                for (i, item) in menu.items_mut().into_iter().enumerate() {
                                    ui.horizontal_top(|ui| {
                                        if i < 31 {
                                            ui.checkbox(&mut enableds[i+1], "Enable item");
                                        } else {
                                            ui.add_enabled(false, egui::widgets::Checkbox::new(&mut true, "Enable item"));
                                        }
                                        
                                        if ui.button("-").clicked() {
                                            to_remove = Some(i);
                                        };
                                        ui.text_edit_singleline(item.text_mut());
                                        ui.label(format!("{:#?}", item.style()));
                                        match item.cfg() {
                                            MenuItemConfig::Plain {
                                                icon,
                                                mut keyboard_shortcut,
                                                mut marking_character,
                                            } => {
                                                let text = if let Some(kbd) = keyboard_shortcut {
                                                    format!("{}", kbd)
                                                } else {
                                                    "Disabled".to_string()
                                                };
                                                egui::ComboBox::from_id_salt(i)
                                                    .selected_text(text)
                                                    .width(80.0)
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut keyboard_shortcut,
                                                            None,
                                                            "Disabled",
                                                        );
                                                        for v in KeyboardShortcut::iter() {
                                                            ui.selectable_value(
                                                                &mut keyboard_shortcut,
                                                                Some(v),
                                                                format!("{}", v),
                                                            );
                                                        }
                                                    });
                                                let marking_text = match marking_character {
                                                    None => "None",
                                                    Some(MarkingCharacter::Checkmark) => {
                                                        "Checkmark"
                                                    }
                                                    Some(MarkingCharacter::EmptyDiamond) => {
                                                        "Empty Diamond"
                                                    }
                                                    Some(MarkingCharacter::FullDiamond) => {
                                                        "Full Diamond"
                                                    }
                                                    Some(MarkingCharacter::Other(_)) => "Other",
                                                };
                                                egui::ComboBox::from_id_salt((i + 1) * 20000)
                                                    .selected_text(marking_text)
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut marking_character,
                                                            None,
                                                            "None",
                                                        );
                                                        ui.selectable_value(
                                                            &mut marking_character,
                                                            Some(MarkingCharacter::Checkmark),
                                                            "Checkmark",
                                                        );
                                                        ui.selectable_value(
                                                            &mut marking_character,
                                                            Some(MarkingCharacter::EmptyDiamond),
                                                            "Empty diamond",
                                                        );
                                                        ui.selectable_value(
                                                            &mut marking_character,
                                                            Some(MarkingCharacter::FullDiamond),
                                                            "Full Diamond",
                                                        );
                                                    });
                                            }
                                            _ => {
                                                ui.label(format!("{:?}", item.cfg()));
                                            }
                                        }
                                    });
                                }

                                let mut bitmask = 0;
                                for (i, enabled) in enableds.into_iter().enumerate() {
                                    if enabled {
                                        bitmask |= 1 << i;
                                    }
                                }
                                menu.set_state(bitmask);

                                if let Some(index) = to_remove {
                                    menu.items_mut().remove(index);
                                }
                                if ui.button("Add item").clicked() {
                                    menu.items_mut().push(MenuItem::new());
                                }
                            }
                            Type::SystemVersion(ver) => {
                                ui.text_edit_singleline(ver.as_mut());
                            }
                            Type::Other(data) => {
                                let mut lines = Vec::new();
                                for (i, chunk) in data.chunks(16).enumerate() {
                                    let mut line = format!("{:08x}:", i * 16);
                                    let mut space = true;
                                    for byte in chunk.iter() {
                                        if space {
                                            line += &format!(" {:02x}", byte);
                                        } else {
                                            line += &format!("{:02x}", byte);
                                        }
                                        space = !space;
                                    }
                                    lines.push(line);
                                }
                                let mut text = lines
                                    .into_iter()
                                    .fold(String::new(), |acc, line| format!("{}{}\n", acc, line));
                                egui::TextEdit::multiline(&mut text)
                                    .code_editor()
                                    .desired_width(f32::INFINITY)
                                    .show(ui);
                            }
                            Type::Version(vers) => {
                                ui.horizontal(|ui| {
                                    ui.label("Version number:");
                                    ui.add(egui::DragValue::new(vers.major_mut()).range(0..=99));
                                    ui.label(".");
                                    let mut minor_hi = (vers.minor() & 0xf0) >> 4;
                                    let mut minor_lo = vers.minor() & 0x0f;
                                    ui.add(egui::DragValue::new(&mut minor_hi).range(0..=9));
                                    ui.label(".");
                                    ui.add(egui::DragValue::new(&mut minor_lo).range(0..=9));
                                    vers.set_minor(minor_lo | (minor_hi << 4));
                                });
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    let text = match vers.development_stage() {
                                        DevelopmentStage::PreAlpha => "Development",
                                        DevelopmentStage::Alpha => "Alpha",
                                        DevelopmentStage::Beta => "Beta",
                                        DevelopmentStage::Released => "Final",
                                    };
                                    ui.label("Release:");
                                    egui::ComboBox::from_id_salt("Release:")
                                        .selected_text(text)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                vers.development_stage_mut(),
                                                DevelopmentStage::PreAlpha,
                                                "Pre-Alpha",
                                            );
                                            ui.selectable_value(
                                                vers.development_stage_mut(),
                                                DevelopmentStage::Alpha,
                                                "Alpha",
                                            );
                                            ui.selectable_value(
                                                vers.development_stage_mut(),
                                                DevelopmentStage::Beta,
                                                "Beta",
                                            );
                                            ui.selectable_value(
                                                vers.development_stage_mut(),
                                                DevelopmentStage::Released,
                                                "Released",
                                            );
                                        });
                                    ui.label("Non-release:");
                                    ui.add(egui::DragValue::new(vers.prerelease_mut()))
                                });
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label("Country code:");
                                    egui::ComboBox::from_id_salt("Country code:")
                                        .selected_text(format!("{}", vers.region_code()))
                                        .show_ui(ui, |ui| {
                                            for code in RegionCode::iter() {
                                                ui.selectable_value(
                                                    vers.region_code_mut(),
                                                    code,
                                                    code.to_string(),
                                                );
                                            }
                                        });
                                });
                                ui.separator();
                                ui.horizontal(|ui| {
                                    ui.label("Short version string:");
                                    ui.text_edit_singleline(vers.version_string_short_mut());
                                });
                                ui.add_space(5.0);
                                ui.label("Long version string (visible in Get Info):");
                                ui.text_edit_multiline(vers.version_string_long_mut());
                            }
                            Type::Code0(code0) => {
                                ui.label(format!("Above A5 Size: {}", code0.above_a5_size()));
                                ui.label(format!("Below A5 Size: {}", code0.below_a5_size()));
                                ui.label("Routines:");
                                for entry in code0.entries() {
                                    ui.label(format!("Offset: {:04x},", entry.routine_offset()));
                                }
                            }
                            Type::Size(size) => {
                                let mut flags = [
                                    size.flags().contains(SizeFlags::SAVE_SCREEN),
                                    size.flags().contains(SizeFlags::ACCEPT_SUSPEND_EVENTS),
                                    size.flags().contains(SizeFlags::DISABLE_OPTION),
                                    size.flags().contains(SizeFlags::CAN_BACKGROUND),
                                    size.flags().contains(SizeFlags::DOES_ACTIVATE_ON_FG_SWITCH),
                                    size.flags().contains(SizeFlags::ONLY_BACKGROUND),
                                    size.flags().contains(SizeFlags::GET_FRONT_CLICKS),
                                    size.flags().contains(SizeFlags::ACCEPT_APP_DIED_EVENTS),
                                    size.flags().contains(SizeFlags::IS_32BIT_COMPATIBLE),
                                    size.flags().contains(SizeFlags::HIGH_LEVEL_EVENT_AWARE),
                                    size.flags()
                                        .contains(SizeFlags::LOCAL_AND_REMOTE_HIGH_LEVEL_EVENTS),
                                    size.flags().contains(SizeFlags::STATIONERY_AWARE),
                                    size.flags().contains(SizeFlags::USE_TEXTEDIT_SERVICES),
                                ];
                                ui.checkbox(&mut flags[0], "Save screen (Obsolete)");
                                ui.checkbox(&mut flags[1], "Accept suspend events");
                                ui.checkbox(&mut flags[2], "Disable option (Obsolete)");
                                ui.checkbox(&mut flags[3], "Can background");
                                ui.checkbox(&mut flags[4], "Does activate on FG switch");
                                ui.checkbox(&mut flags[5], "Only background");
                                ui.checkbox(&mut flags[6], "Get front clicks");
                                ui.checkbox(&mut flags[7], "Accept app died events (debuggers)");
                                ui.checkbox(&mut flags[8], "32 Bit Compatible");
                                ui.checkbox(&mut flags[9], "High level event aware");
                                ui.checkbox(&mut flags[10], "Local and remote high level events");
                                ui.checkbox(&mut flags[11], "Stationery aware");
                                ui.checkbox(&mut flags[12], "Use text edit services");

                                ui.horizontal(|ui| {
                                    ui.label("Size:");
                                    ui.add(egui::DragValue::new(size.preferred_mut()));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Minimum size:");
                                    ui.add(egui::DragValue::new(size.minimum_mut()));
                                });
                            }
                            Type::Template(tmpl) => {
                                ui.label("ResEdit template:");
                                for field in tmpl.fields() {
                                    ui.horizontal(|ui| {
                                        ui.label(field.name());
                                        ui.label(format!("{:?}", field.ty()));
                                    });
                                }
                            }
                            other => {
                                ui.label(format!("Unimplemented data type: {:#x?}", other));
                            }
                        }
                    });
            }
        });
    }
}
