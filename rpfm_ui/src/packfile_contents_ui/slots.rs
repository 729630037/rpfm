//---------------------------------------------------------------------------//
// Copyright (c) 2017-2019 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code related to the main `PackFileContentsSlots`.
!*/

use qt_widgets::file_dialog::{FileDialog, FileMode};
use qt_widgets::slots::SlotQtCorePointRef;
use qt_widgets::tab_widget::TabWidget;
use qt_widgets::tree_view::TreeView;
use qt_widgets::widget::Widget;

use qt_gui::cursor::Cursor;

use qt_core::qt::CaseSensitivity;
use qt_core::slots::{SlotBool, SlotNoArgs, SlotStringRef};

use std::cell::RefCell;
use std::fs::DirBuilder;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rpfm_error::ErrorKind;
use rpfm_lib::common::get_files_from_subdir;
use rpfm_lib::packfile::PathType;
use rpfm_lib::SETTINGS;

use crate::app_ui::AppUI;
use crate::CENTRAL_COMMAND;
use crate::communications::{Command, Response, THREADS_COMMUNICATION_ERROR};
use crate::pack_tree::{icons::IconType, PackTree, TreePathType, TreeViewOperation};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packedfile_views::packfile::PackFileExtraView;
use crate::packedfile_views::{PackedFileView, TheOneSlot};
use crate::QString;
use crate::utils::show_dialog;
use crate::UI_STATE;
use crate::ui_state::op_mode::OperationalMode;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of the PackFile Contents panel.
pub struct PackFileContentsSlots {
    pub open_packedfile_preview: SlotNoArgs<'static>,
    pub open_packedfile_full: SlotNoArgs<'static>,

    pub filter_change_text: SlotStringRef<'static>,
    pub filter_change_autoexpand_matches: SlotBool<'static>,
    pub filter_change_case_sensitive: SlotBool<'static>,

    pub contextual_menu: SlotQtCorePointRef<'static>,
    pub contextual_menu_enabler: SlotNoArgs<'static>,

    pub contextual_menu_add_file: SlotBool<'static>,
    pub contextual_menu_add_folder: SlotBool<'static>,
    pub contextual_menu_add_from_packfile: SlotBool<'static>,
    pub contextual_menu_delete: SlotBool<'static>,
    pub contextual_menu_extract: SlotBool<'static>,
    pub contextual_menu_rename: SlotBool<'static>,

    pub packfile_contents_tree_view_expand_all: SlotNoArgs<'static>,
    pub packfile_contents_tree_view_collapse_all: SlotNoArgs<'static>,
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `PackFileContentsSlots`.
impl PackFileContentsSlots {

	/// This function creates an entire `PackFileContentsSlots` struct.
	pub fn new(app_ui: AppUI, pack_file_contents_ui: PackFileContentsUI, slot_holder: &Rc<RefCell<Vec<TheOneSlot>>>) -> Self {

        // Slot to open the selected PackedFile as a preview.
        let open_packedfile_preview = SlotNoArgs::new(clone!(slot_holder => move || {
            app_ui.open_packedfile(&pack_file_contents_ui, &slot_holder, true);
        }));

        // Slot to open the selected PackedFile as a permanent view.
        let open_packedfile_full = SlotNoArgs::new(clone!(slot_holder => move || {
            app_ui.open_packedfile(&pack_file_contents_ui, &slot_holder, false);
        }));

        // What happens when we trigger one of the filter events for the PackFile Contents TreeView.
        let filter_change_text = SlotStringRef::new(move |_| {
            pack_file_contents_ui.filter_files();
        });
        let filter_change_autoexpand_matches = SlotBool::new(move |_| {
            pack_file_contents_ui.filter_files();
        });
        let filter_change_case_sensitive = SlotBool::new(move |_| {
            pack_file_contents_ui.filter_files();
        });

        // Slot to show the Contextual Menu for the TreeView.
        let contextual_menu = SlotQtCorePointRef::new(move |_| {
            unsafe { pack_file_contents_ui.packfile_contents_tree_view_context_menu.as_mut().unwrap().exec2(&Cursor::pos()); }
        });

        // Slot to enable/disable contextual actions depending on the selected item.
        let contextual_menu_enabler = SlotNoArgs::new(move || {
                let (contents, files, folders) = <*mut TreeView as PackTree>::get_combination_from_main_treeview_selection(&pack_file_contents_ui);
                match contents {

                    // Only one or more files selected.
                    1 => {

                        // These options are valid for 1 or more files.
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }

                        // These options are limited to only 1 file selected, and should not be usable if multiple files
                        // are selected.
                        let enabled = if files == 1 { true } else { false };
                        unsafe {
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(enabled);
                        }

                        // Only if we have multiple files selected, we give the option to merge. Further checks are done when clicked.
                        let enabled = if files > 1 { true } else { false };
                        unsafe { pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(enabled); }
                    },

                    // Only one or more folders selected.
                    2 => {

                        // These options are valid for 1 or more folders.
                        unsafe {
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }

                        // These options are limited to only 1 folder selected.
                        let enabled = if folders == 1 { true } else { false };
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(enabled);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(enabled);
                        }
                    },

                    // One or more files and one or more folders selected.
                    3 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // One PackFile (you cannot have two in the same TreeView) selected.
                    4 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // PackFile and one or more files selected.
                    5 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // PackFile and one or more folders selected.
                    6 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // PackFile, one or more files, and one or more folders selected.
                    7 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(true);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(true);
                        }
                    },

                    // No paths selected, none selected, invalid path selected, or invalid value.
                    0 | 8..=255 => {
                        unsafe {
                            pack_file_contents_ui.context_menu_add_file.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_add_from_packfile.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_loc.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_create_text.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_merge_tables.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_delete.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_extract.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_rename.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_decoder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_dependency_manager.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_containing_folder.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_with_external_program.as_mut().unwrap().set_enabled(false);
                            pack_file_contents_ui.context_menu_open_notes.as_mut().unwrap().set_enabled(false);
                        }
                    },
                }

                // Ask the other thread if there is a Dependency Database and a Schema loaded.
                CENTRAL_COMMAND.send_message_qt(Command::IsThereADependencyDatabase);
                CENTRAL_COMMAND.send_message_qt(Command::IsThereASchema);
                let is_there_a_dependency_database = match CENTRAL_COMMAND.recv_message_qt() {
                    Response::Bool(it_is) => it_is,
                    _ => panic!(THREADS_COMMUNICATION_ERROR),
                };

                let is_there_a_schema = match CENTRAL_COMMAND.recv_message_qt() {
                    Response::Bool(it_is) => it_is,
                    _ => panic!(THREADS_COMMUNICATION_ERROR),
                };

                // If there is no dependency_database or schema for our GameSelected, ALWAYS disable creating new DB Tables and exporting them.
                if !is_there_a_dependency_database || !is_there_a_schema {
                    unsafe { pack_file_contents_ui.context_menu_check_tables.as_mut().unwrap().set_enabled(false); }
                    unsafe { pack_file_contents_ui.context_menu_create_db.as_mut().unwrap().set_enabled(false); }
                    unsafe { pack_file_contents_ui.context_menu_mass_import_tsv.as_mut().unwrap().set_enabled(false); }
                    unsafe { pack_file_contents_ui.context_menu_mass_export_tsv.as_mut().unwrap().set_enabled(false); }
                }
            }
        );

        // What happens when we trigger the "Add File/s" action in the Contextual Menu.
        let contextual_menu_add_file = SlotBool::new(move |_| {

                // Create the FileDialog to get the file/s to add and configure it.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.main_window as *mut Widget,
                    &QString::from_std_str("Add File/s"),
                )) };
                file_dialog.set_file_mode(FileMode::ExistingFiles);
                match UI_STATE.get_operational_mode() {

                    // If we have a "MyMod" selected...
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let mymods_base_path = &SETTINGS.lock().unwrap().paths["mymods_base_path"];
                        if let Some(ref mymods_base_path) = mymods_base_path {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());
                            file_dialog.set_directory(&QString::from_std_str(assets_folder.to_string_lossy().to_owned()));

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() && DirBuilder::new().recursive(true).create(&assets_folder).is_err() {
                                return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::IOCreateAssetFolder, false);
                            }

                            // Run it and expect a response (1 => Accept, 0 => Cancel).
                            if file_dialog.exec() == 1 {

                                // Get the Paths of the files we want to add.
                                let mut paths: Vec<PathBuf> = vec![];
                                let paths_qt = file_dialog.selected_files();
                                for index in 0..paths_qt.size() { paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                                // Check if the files are in the Assets Folder. The file chooser kinda guarantees that
                                // all are in the same folder, so we can just check the first one.
                                let paths_packedfile: Vec<Vec<String>> = if paths[0].starts_with(&assets_folder) {
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths {
                                        let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                                        paths_packedfile.push(filtered_path.iter().map(|x| x.to_string_lossy().as_ref().to_owned()).collect::<Vec<String>>());
                                    }
                                    paths_packedfile
                                }

                                // Otherwise, they are added like normal files.
                                else {
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&pack_file_contents_ui, &path, true)); }
                                    paths_packedfile
                                };

                                pack_file_contents_ui.add_packedfiles(&app_ui, &paths, &paths_packedfile);
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::MyModPathNotConfigured, false); }
                    }

                    // If it's in "Normal" mode...
                    OperationalMode::Normal => {

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 {

                            // Get the Paths of the files we want to add.
                            let mut paths: Vec<PathBuf> = vec![];
                            let paths_qt = file_dialog.selected_files();
                            for index in 0..paths_qt.size() { paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                            // Get their final paths in the PackFile and only proceed if all of them are closed.
                            let mut paths_packedfile: Vec<Vec<String>> = vec![];
                            for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&pack_file_contents_ui, &path, true)); }

                            pack_file_contents_ui.add_packedfiles(&app_ui, &paths, &paths_packedfile);
                        }
                    }
                }
            }
        );

        // What happens when we trigger the "Add Folder/s" action in the Contextual Menu.
        let contextual_menu_add_folder = SlotBool::new(move |_| {

                // Create the FileDialog to get the folder/s to add and configure it.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.main_window as *mut Widget,
                    &QString::from_std_str("Add Folder/s"),
                )) };
                file_dialog.set_file_mode(FileMode::Directory);
                match UI_STATE.get_operational_mode() {

                    // If we have a "MyMod" selected...
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {

                        // In theory, if we reach this line this should always exist. In theory I should be rich.
                        let mymods_base_path = &SETTINGS.lock().unwrap().paths["mymods_base_path"];
                        if let Some(ref mymods_base_path) = mymods_base_path {

                            // We get the assets folder of our mod (without .pack extension).
                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(Path::new(&mod_name).file_stem().unwrap().to_string_lossy().as_ref().to_owned());
                            file_dialog.set_directory(&QString::from_std_str(assets_folder.to_string_lossy().to_owned()));

                            // We check that path exists, and create it if it doesn't.
                            if !assets_folder.is_dir() && DirBuilder::new().recursive(true).create(&assets_folder).is_err() {
                                return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::IOCreateAssetFolder, false);
                            }

                            // Run it and expect a response (1 => Accept, 0 => Cancel).
                            if file_dialog.exec() == 1 {

                                // Get the Paths of the folders we want to add.
                                let mut folder_paths: Vec<PathBuf> = vec![];
                                let paths_qt = file_dialog.selected_files();
                                for index in 0..paths_qt.size() { folder_paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                                // Get the Paths of the files inside the folders we want to add.
                                let mut paths: Vec<PathBuf> = vec![];
                                for path in &folder_paths { paths.append(&mut get_files_from_subdir(&path).unwrap()); }

                                // Check if the files are in the Assets Folder. All are in the same folder, so we can just check the first one.
                                let paths_packedfile = if paths[0].starts_with(&assets_folder) {
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths {
                                        let filtered_path = path.strip_prefix(&assets_folder).unwrap();
                                        paths_packedfile.push(filtered_path.iter().map(|x| x.to_string_lossy().as_ref().to_owned()).collect::<Vec<String>>());
                                    }
                                    paths_packedfile
                                }

                                // Otherwise, they are added like normal files.
                                else {
                                    let mut paths_packedfile: Vec<Vec<String>> = vec![];
                                    for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&pack_file_contents_ui, &path, true)); }
                                    paths_packedfile
                                };

                                pack_file_contents_ui.add_packedfiles(&app_ui, &paths, &paths_packedfile);
                            }
                        }

                        // If there is no "MyMod" path configured, report it.
                        else { return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::MyModPathNotConfigured, false); }
                    }

                    // If it's in "Normal" mode, we just get the paths of the files inside them and add those files.
                    OperationalMode::Normal => {

                        // Run it and expect a response (1 => Accept, 0 => Cancel).
                        if file_dialog.exec() == 1 {

                            // Get the Paths of the folders we want to add.
                            let mut folder_paths: Vec<PathBuf> = vec![];
                            let paths_qt = file_dialog.selected_files();
                            for index in 0..paths_qt.size() { folder_paths.push(PathBuf::from(paths_qt.at(index).to_std_string())); }

                            // Get the Paths of the files inside the folders we want to add.
                            let mut paths: Vec<PathBuf> = vec![];
                            for path in &folder_paths { paths.append(&mut get_files_from_subdir(&path).unwrap()); }

                            // Get their final paths in the PackFile and only proceed if all of them are closed.
                            let mut paths_packedfile: Vec<Vec<String>> = vec![];
                            for path in &paths { paths_packedfile.append(&mut <*mut TreeView as PackTree>::get_path_from_pathbuf(&pack_file_contents_ui, &path, true)); }
                            pack_file_contents_ui.add_packedfiles(&app_ui, &paths, &paths_packedfile);
                        }
                    }
                }
            }
        );

        // What happens when we trigger the "Add From PackFile" action in the Contextual Menu.
        let contextual_menu_add_from_packfile = SlotBool::new(clone!(
            slot_holder => move |_| {

                // Create the FileDialog to get the PackFile to open, configure it and run it.
                let mut file_dialog = unsafe { FileDialog::new_unsafe((
                    app_ui.main_window as *mut Widget,
                    &QString::from_std_str("Select PackFile"),
                )) };

                file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
                if file_dialog.exec() == 1 {
                    let path_str = file_dialog.selected_files().at(0).to_std_string();
                    let path = PathBuf::from(path_str.to_owned());
                    unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }

                    let mut tab = PackedFileView::default();
                    match PackFileExtraView::new_view(&mut tab, &app_ui, &pack_file_contents_ui, path) {
                        Ok(slots) => {
                            slot_holder.borrow_mut().push(slots);

                            // Add the file to the 'Currently open' list and make it visible.
                            let tab_widget = tab.get_mut_widget();
                            let name = path_str;
                            let icon_type = IconType::PackFile(false);
                            let icon = icon_type.get_icon_from_path();

                            // If there is another Extra PackFile already open, close it.
                            {
                                let open_packedfiles = UI_STATE.set_open_packedfiles();
                                if let Some(view) = open_packedfiles.get(&vec!["extra_packfile.rpfm_reserved".to_owned()]) {
                                    let widget = view.get_mut_widget();
                                    let index = unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().index_of(widget) };

                                    unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().remove_tab(index); }
                                }
                            }
                            app_ui.purge_that_one_specifically(&["extra_packfile.rpfm_reserved".to_owned()], false);

                            unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().add_tab((tab_widget, icon, &QString::from_std_str(&name))); }
                            unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().set_current_widget(tab_widget); }
                            let mut open_list = UI_STATE.set_open_packedfiles();
                            open_list.insert(vec!["packfile_extra.rpfm_reserved".to_owned()], tab);

                        }
                        Err(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                    }
                    unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
                }
            }
        ));

        // What happens when we trigger the "Delete" action in the Contextual Menu.
        let contextual_menu_delete = SlotBool::new(clone!(
            slot_holder => move |_| {
                let selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                let selected_items = selected_items.iter().map(|x| From::from(x)).collect::<Vec<PathType>>();

                CENTRAL_COMMAND.send_message_qt(Command::DeletePackedFiles(selected_items));
                match CENTRAL_COMMAND.recv_message_qt() {
                    Response::VecPathType(deleted_items) => {
                        let items = deleted_items.iter().map(|x| From::from(x)).collect::<Vec<TreePathType>>();
                        pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Delete(items.to_vec()));

                        // Remove all the deleted PackedFiles from the cache.
                        for item in &items {
                            match item {
                                TreePathType::File(path) => app_ui.purge_that_one_specifically(path, false),
                                TreePathType::Folder(path) => {
                                    let mut paths_to_remove = vec![];
                                    {
                                        let open_packedfiles = UI_STATE.set_open_packedfiles();
                                        for packed_file_path in open_packedfiles.keys() {
                                            if !packed_file_path.is_empty() && packed_file_path.starts_with(path) {
                                                paths_to_remove.push(packed_file_path.to_vec());
                                            }
                                        }
                                    }

                                    for path in paths_to_remove {
                                        app_ui.purge_that_one_specifically(&path, false);
                                    }

                                }
                                TreePathType::PackFile => app_ui.purge_them_all(&slot_holder),
                                TreePathType::None => unreachable!(),
                            }
                        }
                    },
                    _ => panic!(THREADS_COMMUNICATION_ERROR),
                };
            }
        ));

        // What happens when we trigger the "Extract" action in the Contextual Menu.
        let contextual_menu_extract = SlotBool::new(move |_| {

                // Get the currently selected paths (and visible) paths.
                let selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                let selected_items = selected_items.iter().map(|x| From::from(x)).collect::<Vec<PathType>>();
                let extraction_path = match UI_STATE.get_operational_mode() {

                    // In MyMod mode we extract directly to the folder of the selected MyMod, keeping the folder structure.
                    OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                        if let Some(ref mymods_base_path) = SETTINGS.lock().unwrap().paths["mymods_base_path"] {

                            // We get the assets folder of our mod (without .pack extension). This mess removes the .pack.
                            let mut mod_name = mod_name.to_owned();
                            mod_name.pop();
                            mod_name.pop();
                            mod_name.pop();
                            mod_name.pop();
                            mod_name.pop();

                            let mut assets_folder = mymods_base_path.to_path_buf();
                            assets_folder.push(&game_folder_name);
                            assets_folder.push(&mod_name);
                            assets_folder
                        }

                        // If there is no MyMod path configured, report it.
                        else { return show_dialog(app_ui.main_window as *mut Widget, ErrorKind::MyModPathNotConfigured, true); }
                    }

                    // In normal mode, we ask the user to provide us with a path.
                    OperationalMode::Normal => {
                        let extraction_path = unsafe { FileDialog::get_existing_directory_unsafe((
                            app_ui.main_window as *mut Widget,
                            &QString::from_std_str("Extract PackFile"),
                        )) };

                        if !extraction_path.is_empty() { PathBuf::from(extraction_path.to_std_string()) }
                        else { return }
                    }
                };

                // We have to save our data from cache to the backend before extracting it. Otherwise we would extract outdated data.
                // TODO: Make this more... optimal.
                UI_STATE.get_open_packedfiles().iter().for_each(|(path, packed_file)| packed_file.save(path));

                CENTRAL_COMMAND.send_message_qt(Command::ExtractPackedFiles(selected_items, extraction_path));
                unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(false); }
                match CENTRAL_COMMAND.recv_message_qt() {
                    Response::String(result) => show_dialog(app_ui.main_window as *mut Widget, result, true),
                    Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                    _ => panic!(THREADS_COMMUNICATION_ERROR),
                }
                unsafe { (app_ui.main_window.as_mut().unwrap() as &mut Widget).set_enabled(true); }
            }
        );


        // What happens when we trigger the "Rename" Action.
        let contextual_menu_rename = SlotBool::new(move |_| {

                // Get the currently selected items, and check how many of them are valid before trying to rewrite them.
                // Why? Because I'm sure there is an asshole out there that it's going to try to give the files duplicated
                // names, and if that happen, we have to stop right there that criminal scum.
                let selected_items = <*mut TreeView as PackTree>::get_item_types_from_main_treeview_selection(&pack_file_contents_ui);
                if let Some(rewrite_sequence) = PackFileContentsUI::create_rename_dialog(&app_ui, &selected_items) {
                    let mut renaming_data_background: Vec<(PathType, String)> = vec![];
                    for item_type in selected_items {
                        match item_type {
                            TreePathType::File(ref path) | TreePathType::Folder(ref path) => {
                                let original_name = path.last().unwrap();
                                let new_name = rewrite_sequence.to_owned().replace("{x}", &original_name);
                                renaming_data_background.push((From::from(&item_type), new_name));

                            },

                            // These two should, if everything works properly, never trigger.
                            TreePathType::PackFile | TreePathType::None => unimplemented!(),
                        }
                    }

                    // Send the renaming data to the Background Thread, wait for a response.
                    CENTRAL_COMMAND.send_message_qt(Command::RenamePackedFiles(renaming_data_background));
                    match CENTRAL_COMMAND.recv_message_qt() {
                        Response::VecPathTypeString(renamed_items) => {
                            let renamed_items = renamed_items.iter().map(|x| (From::from(&x.0), x.1.to_owned())).collect::<Vec<(TreePathType, String)>>();

                            let mut path_changes = vec![];
                            let mut open_packedfiles = UI_STATE.set_open_packedfiles();
                            for (path, _) in open_packedfiles.iter_mut() {
                                if !path.is_empty() {
                                    for (item_type, new_name) in &renamed_items {
                                        match item_type {
                                            TreePathType::File(ref item_path) => {
                                                if item_path == path {

                                                    // Get the new path.
                                                    let mut new_path = item_path.to_vec();
                                                    *new_path.last_mut().unwrap() = new_name.to_owned();
                                                    path_changes.push((path.to_vec(), new_path.to_vec()));

                                                    // Update the global search stuff, if needed.
                                                    //global_search_explicit_paths.borrow_mut().append(&mut vec![new_path; 1]);
                                                }
                                            }

                                            TreePathType::Folder(ref item_path) => {
                                                if !item_path.is_empty() && path.starts_with(&item_path) {

                                                    let mut new_folder_path = item_path.to_vec();
                                                    *new_folder_path.last_mut().unwrap() = new_name.to_owned();

                                                    let mut new_file_path = new_folder_path.to_vec();
                                                    new_file_path.append(&mut (&path[item_path.len()..]).to_vec());
                                                    path_changes.push((path.to_vec(), new_file_path.to_vec()));

                                                    // Update the global search stuff, if needed.
                                                    //global_search_explicit_paths.borrow_mut().append(&mut vec![new_folder_path; 1]);
                                                }
                                            }
                                            _ => unreachable!(),
                                        }
                                    }
                                }
                            }

                            for (path_before, path_after) in &path_changes {
                                let data = open_packedfiles.remove(path_before).unwrap();
                                let widget = data.get_mut_widget();
                                let index = unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().index_of(widget) };
                                let mut text = unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().tab_text(index) };
                                let old_name = path_before.last().unwrap();
                                let new_name = path_after.last().unwrap();
                                text.replace((&QString::from_std_str(old_name), &QString::from_std_str(new_name), CaseSensitivity::Sensitive));
                                unsafe { app_ui.tab_bar_packed_file.as_mut().unwrap().set_tab_text(index, &text); }
                                open_packedfiles.insert(path_after.to_vec(), data);
                            }

                            pack_file_contents_ui.packfile_contents_tree_view.update_treeview(true, TreeViewOperation::Rename(renamed_items));
                        },
                        Response::Error(error) => show_dialog(app_ui.main_window as *mut Widget, error, false),
                        _ => panic!(THREADS_COMMUNICATION_ERROR),
                    }
                }
            }
        );


        let packfile_contents_tree_view_expand_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().expand_all(); }});
        let packfile_contents_tree_view_collapse_all = SlotNoArgs::new(move || { unsafe { pack_file_contents_ui.packfile_contents_tree_view.as_mut().unwrap().collapse_all(); }});

        // And here... we return all the slots.
		Self {
            open_packedfile_preview,
            open_packedfile_full,

            filter_change_text,
            filter_change_autoexpand_matches,
            filter_change_case_sensitive,

            contextual_menu,
            contextual_menu_enabler,

            contextual_menu_add_file,
            contextual_menu_add_folder,
            contextual_menu_add_from_packfile,
            contextual_menu_delete,
            contextual_menu_extract,
            contextual_menu_rename,

            packfile_contents_tree_view_expand_all,
            packfile_contents_tree_view_collapse_all,
		}
	}
}