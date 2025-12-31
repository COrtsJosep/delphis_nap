use crate::modules::financial::*;
use crate::modules::gui::{AppState, WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui;
use eframe::egui::{Color32, ComboBox};
use egui::{containers, Align, Layout, PopupCloseBehavior};
use egui_autocomplete::AutoCompleteTextEdit;
use egui_extras::*;
use strum::IntoEnumIterator;

impl AppState {
    fn clear_fields(&mut self) -> () {
        *self = AppState::default();
    }

    fn clear_transaction_fields(&mut self) -> () {
        self.transaction_category = String::default();
        self.transaction_subcategory = String::default();
        self.transaction_description = String::default();
        self.transaction_entity_id = i64::default();
        self.transaction_entity_string = String::default();
        self.transaction_account_id = i64::default();
        self.transaction_account_string = String::default();
        self.transaction_type = TransactionType::default();
        self.transaction_filter = String::default();
    }

    fn clear_entity_fields(&mut self) -> () {
        self.entity_name = String::default();
        self.entity_country = String::default();
        self.entity_type = EntityType::default();
        self.entity_subtype = String::default();
    }

    fn clear_account_fields(&mut self) -> () {
        self.account_name = String::default();
        self.account_country = String::default();
        self.account_currency = Currency::default();
        self.account_type = AccountType::default();
        self.account_initial_balance = f64::default();
        self.account_initial_balance_tentative = String::default();
    }

    fn are_valid_entity_fields(&self) -> bool {
        (self.entity_name.len() > 0) & (self.entity_country.len() > 0)
    }

    fn is_valid_initial_balance(&self) -> bool {
        let parsing_result = self.account_initial_balance_tentative.parse::<f64>();
        match parsing_result {
            Ok(_value) => true,
            Err(_e) => false,
        }
    }

    fn are_valid_account_fields(&self) -> bool {
        (self.account_name.len() > 0)
            & (self.account_country.len() > 0)
            & self.is_valid_initial_balance()
    }

    fn is_valid_transaction_value(&self) -> bool {
        let parsing_result = self.transaction_value_tentative.parse::<f64>();
        match parsing_result {
            Ok(_value) => true,
            Err(_e) => false,
        }
    }

    fn is_valid_transaction_currency(&mut self) -> bool {
        match self.database.account(self.transaction_account_id) {
            Ok(account) => &self.transaction_currency == account.currency(),
            Err(e) => {
                self.throw_error(e);
                false
            }
        }
    }

    fn are_valid_transaction_fields(&mut self) -> bool {
        ((self.transaction_category.len() > 0)
            | (self.transaction_type.is_fund_change() & self.is_valid_transaction_currency()))
            & self.is_valid_transaction_value()
    }

    pub fn handle_show_input_entity_window(&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("input_entity_window"),
            egui::ViewportBuilder::default()
                .with_title("Input entity window")
                .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::Grid::new("my_grid")
                        .num_columns(3)
                        .spacing([45.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Entity name:")
                                .on_hover_text("Name of the entity. For instance, the shop's name");
                            ui.text_edit_singleline(&mut self.entity_name);
                            if self.entity_name.len() > 0 {
                                ui.colored_label(
                                    Color32::from_rgb(110, 255, 110),
                                    "Valid entity name!",
                                );
                            } else {
                                ui.colored_label(
                                    Color32::from_rgb(255, 0, 0),
                                    "Please enter an entity name!",
                                );
                            }
                            ui.end_row();

                            ui.label("Entity country:")
                                .on_hover_text("Country where the entity is based.");
                            match self.database.entity_countries() {
                                Ok(entity_countries) => {
                                    ui.add(
                                        AutoCompleteTextEdit::new(
                                            &mut self.entity_country,
                                            entity_countries,
                                        )
                                        .max_suggestions(10)
                                        .highlight_matches(true),
                                    );
                                }
                                Err(e) => {
                                    self.throw_polars_error(e);
                                }
                            }
                            if self.entity_country.len() > 0 {
                                ui.colored_label(
                                    Color32::from_rgb(110, 255, 110),
                                    "Valid entity country!",
                                );
                            } else {
                                ui.colored_label(
                                    Color32::from_rgb(255, 0, 0),
                                    "Please enter an entity country!",
                                );
                            }
                            ui.end_row();

                            ui.label("Entity type:")
                                .on_hover_text("Category of the entity.");
                            ComboBox::from_id_salt("Entity type")
                                .selected_text(format!("{}", self.entity_type))
                                .show_ui(ui, |ui| {
                                    for possible_entity_type in EntityType::iter() {
                                        ui.selectable_value(
                                            &mut self.entity_type,
                                            possible_entity_type.clone(),
                                            format!("{possible_entity_type}"),
                                        );
                                    }
                                });
                            ui.end_row();

                            ui.label("Entity subtype:")
                                .on_hover_text("Sub-category of the entity.");
                            match self.database.entity_subtypes() {
                                Ok(entity_subtypes) => {
                                    ui.add(
                                        AutoCompleteTextEdit::new(
                                            &mut self.entity_subtype,
                                            entity_subtypes,
                                        )
                                        .max_suggestions(10)
                                        .highlight_matches(true),
                                    );
                                }
                                Err(e) => {
                                    self.throw_polars_error(e);
                                }
                            }
                            ui.end_row();
                        });

                    ui.separator();
                    ui.vertical_centered_justified(|ui| {
                        if self.are_valid_entity_fields() {
                            if ui.button("Add new entity").clicked() {
                                let entity: Entity = Entity::new(
                                    self.entity_name.clone(),
                                    self.entity_country.clone(),
                                    self.entity_type.clone(),
                                    self.entity_subtype.clone(),
                                );
                                match self.database.insert_entity(&entity) {
                                    Ok(entity_id) => {
                                        self.transaction_entity_id = entity_id;
                                        self.database.save();
                                        self.clear_entity_fields();

                                        self.show_input_entity_window = false;
                                    }
                                    Err(e) => {
                                        self.throw_polars_error(e);
                                    }
                                }
                            }
                        }
                    });
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_input_entity_window = false;
                }
            },
        );
    }
    pub fn handle_show_input_account_window(&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("input_account_window"),
            egui::ViewportBuilder::default()
                .with_title("Input account window")
                .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::Grid::new("my_grid")
                        .num_columns(3)
                        .spacing([45.0, 4.0])
                        //.striped(true)
                        .show(ui, |ui| {
                            ui.label("Account name: ").on_hover_text("Name of the account. For instance, the name of the bank, or the investment fund.");
                            ui.text_edit_singleline(&mut self.account_name);
                            if self.account_name.len() > 0 {
                                ui.colored_label(
                                    Color32::from_rgb(110, 255, 110),
                                    "Valid account name!",
                                );
                            } else {
                                ui.colored_label(
                                    Color32::from_rgb(255, 0, 0),
                                    "Invalid account name!",
                                );
                            }
                            ui.end_row();

                            ui.label("Account country: ").on_hover_text("Country where the account is based.");
                            match self.database.account_countries() {
                                Ok(account_countries) => {
                            ui.add(
                                AutoCompleteTextEdit::new(
                                    &mut self.account_country,
                                    account_countries,
                                )
                                    .max_suggestions(10)
                                    .highlight_matches(true),
                            );}, 
                                Err(e) => {self.throw_error(e);}
                            }
                            if self.account_country.len() > 0 {
                                ui.colored_label(
                                    Color32::from_rgb(110, 255, 110),
                                    "Valid account country!",
                                );
                            } else {
                                ui.colored_label(
                                    Color32::from_rgb(255, 0, 0),
                                    "Invalid account country!",
                                );
                            }
                            ui.end_row();

                            ui.label("Account currency: ").on_hover_text("Currency of the account. If multiple, consider creating various accounts with different currencies.");
                            ComboBox::from_id_salt("Account currency")
                                .selected_text(format!("{}", self.account_currency))
                                .show_ui(ui, |ui| {
                                    for possible_account_currency in Currency::iter() {
                                        ui.selectable_value(
                                            &mut self.account_currency,
                                            possible_account_currency.clone(),
                                            format!("{possible_account_currency}"),
                                        );
                                    }
                                });
                            ui.end_row();

                            ui.label("Account type: ").on_hover_text("Category of the account.");
                            ComboBox::from_id_salt("Account type")
                                .selected_text(format!("{}", self.account_type))
                                .show_ui(ui, |ui| {
                                    for possible_account_type in AccountType::iter() {
                                        ui.selectable_value(
                                            &mut self.account_type,
                                            possible_account_type.clone(),
                                            format!("{possible_account_type}"),
                                        );
                                    }
                                });
                            ui.end_row();

                            ui.label("Account initial balance: ").on_hover_text("Amount of money stored in the account, in the given currency, in this very moment.");
                            ui.text_edit_singleline(&mut self.account_initial_balance_tentative);
                            if self.is_valid_initial_balance() {
                                ui.colored_label(
                                    Color32::from_rgb(110, 255, 110),
                                    "Valid initial balance!",
                                );
                            } else {
                                ui.colored_label(
                                    Color32::from_rgb(255, 0, 0),
                                    "Invalid initial balance!",
                                );
                            }
                            ui.end_row();
                        });

                     ui.separator();
                    ui.vertical_centered_justified(|ui| {
                       if self.are_valid_account_fields() {
                            self.account_initial_balance = self
                                .account_initial_balance_tentative
                                .parse::<f64>()
                                .expect("Error parsing account initial balance");

                            if ui.button("Add new account").on_hover_text("Save account into the database.").clicked() {
                                let account: Account = Account::new(
                                    self.account_name.clone(),
                                    self.account_country.clone(),
                                    self.account_currency.clone(),
                                    self.account_type.clone(),
                                    self.account_initial_balance,
                                );

                                match self.database.insert_account(&account) {
                                    Ok(account_id) => {

                                self.transaction_account_id = account_id;
                                self.database.save();
                                self.clear_account_fields();

                                self.show_input_account_window = false;}, Err(e) => {self.throw_polars_error(e)}}
                            }
                        }
                    });
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_input_account_window = false;
                }
            },
        );
    }
    pub fn handle_show_input_party_window(&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("input_party_window"),
            egui::ViewportBuilder::default()
                .with_title("Party Creation")
                .with_inner_size([WINDOW_WIDTH / 1.5, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    StripBuilder::new(ui)
                        .size(Size::exact(40.0))
                        .size(Size::remainder().at_least(100.0))
                        .size(Size::exact(40.0))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    if ui.button("Add new transaction").clicked() {
                                        self.show_input_transaction_window =
                                            self.show_input_party_window & true;
                                    }
                                });
                                ui.separator();
                            });
                            strip.cell(|ui| {
                                TableBuilder::new(ui)
                                    .columns(Column::auto().resizable(true).at_least(50.0), 5)
                                    .striped(true)
                                    .cell_layout(Layout::right_to_left(Align::Center))
                                    .header(20.0, |mut header| {
                                        for column_name in [
                                            "Transaction Type",
                                            "Value",
                                            "Currency",
                                            "Date",
                                            "Action",
                                        ] {
                                            header.col(|ui| {
                                                ui.strong(column_name).on_hover_text(column_name);
                                            });
                                        }
                                    })
                                    .body(|mut body| {
                                        if self.party.is_empty() {
                                            body.row(30.0, |mut row| {
                                                row.col(|ui| {
                                                    ui.label("Nothing");
                                                });
                                                row.col(|ui| {
                                                    ui.label("to");
                                                });
                                                row.col(|ui| {
                                                    ui.label("show");
                                                });
                                                row.col(|ui| {
                                                    ui.label("here");
                                                });
                                                row.col(|ui| {
                                                    ui.label("yet");
                                                });
                                            })
                                        }

                                        let mut i_remove = 0;
                                        let mut remove = false;
                                        for (i, transaction) in self.party.iter().enumerate() {
                                            body.row(30.0, |mut row| {
                                                row.col(|ui| {
                                                    ui.label(transaction.transaction_type());
                                                });
                                                row.col(|ui| {
                                                    ui.label(format!("{:.2}", transaction.value()));
                                                });
                                                row.col(|ui| {
                                                    ui.label(transaction.currency().to_string());
                                                });
                                                row.col(|ui| {
                                                    ui.label(transaction.date().to_string());
                                                });
                                                row.col(|ui| {
                                                    if ui
                                                        .button("Edit/Remove")
                                                        .on_hover_text(
                                                            "Removes transaction from the party",
                                                        )
                                                        .clicked()
                                                    {
                                                        i_remove = i;
                                                        remove = true;
                                                    }
                                                });
                                            });
                                        }
                                        if remove {
                                            let removed_transaction: Transaction =
                                                self.party.remove(i_remove);

                                            match removed_transaction {
                                                // unpack the attributes of
                                                // the transaction
                                                Transaction::Income {
                                                    value,
                                                    currency,
                                                    date,
                                                    category,
                                                    subcategory,
                                                    description,
                                                    entity_id,
                                                } => {
                                                    self.transaction_type = TransactionType::Income;
                                                    self.transaction_value = value;
                                                    self.transaction_value_tentative =
                                                        value.to_string();
                                                    self.transaction_currency = currency;
                                                    self.transaction_date = date;
                                                    self.transaction_category = category;
                                                    self.transaction_subcategory = subcategory;
                                                    self.transaction_description = description;
                                                    self.transaction_entity_id = entity_id;
                                                }
                                                Transaction::Expense {
                                                    value,
                                                    currency,
                                                    date,
                                                    category,
                                                    subcategory,
                                                    description,
                                                    entity_id,
                                                } => {
                                                    self.transaction_type =
                                                        TransactionType::Expense;
                                                    self.transaction_value = value;
                                                    self.transaction_value_tentative =
                                                        value.to_string();
                                                    self.transaction_currency = currency;
                                                    self.transaction_date = date;
                                                    self.transaction_category = category;
                                                    self.transaction_subcategory = subcategory;
                                                    self.transaction_description = description;
                                                    self.transaction_entity_id = entity_id;
                                                }
                                                Transaction::Credit {
                                                    value,
                                                    currency,
                                                    date,
                                                    account_id,
                                                } => {
                                                    self.transaction_type = TransactionType::Credit;
                                                    self.transaction_value = value;
                                                    self.transaction_value_tentative =
                                                        value.to_string();
                                                    self.transaction_currency = currency;
                                                    self.transaction_date = date;
                                                    self.transaction_account_id = account_id;
                                                }
                                                Transaction::Debit {
                                                    value,
                                                    currency,
                                                    date,
                                                    account_id,
                                                } => {
                                                    self.transaction_type = TransactionType::Debit;
                                                    self.transaction_value = value;
                                                    self.transaction_value_tentative =
                                                        value.to_string();
                                                    self.transaction_currency = currency;
                                                    self.transaction_date = date;
                                                    self.transaction_account_id = account_id;
                                                }
                                            }

                                            self.show_input_transaction_window = true;
                                        }
                                    });
                            });
                            strip.cell(|ui| {
                                ui.separator();
                                ui.vertical_centered_justified(|ui| {
                                    if self.party.is_valid() {
                                        if ui.button("Add party").clicked() {
                                            match self.database.insert_party(&mut self.party) {
                                                Ok(_) => {
                                                    match self.database.save() { 
                                                        Ok(_) => {
                                                            self.clear_fields();
                                                            self.show_input_party_window = false;
                                                        },
                                                        Err(e) => {self.throw_error(e);}
                                                    }}, 
                                                Err(e) => {self.throw_error(e);}
                                                }
                                        }
                                    }
                                });
                            });
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.clear_fields();
                    self.show_input_party_window = false;
                }
            },
        );
    }
    pub fn handle_show_input_transaction_window(&mut self, ctx: &egui::Context) -> () {
        match self.database.entity(self.transaction_entity_id) {
            Ok(transaction_entity) => {
        self.transaction_entity_string =
            transaction_entity.to_string();}, Err(e) => {self.throw_error(e);}}
        match self
            .database
            .account(self.transaction_account_id) { 
        Ok(account) => {self.transaction_account_string = account.to_string();}, Err(e) => {self.throw_error(e);}}

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("input_transaction_window"),
            egui::ViewportBuilder::default()
                .with_title("Input transaction window")
                .with_inner_size([WINDOW_WIDTH * 1.1, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::Grid::new("my_grid")
                        .num_columns(3)
                        .spacing([45.0, 4.0])
                        .min_col_width(150.0)
                        //.striped(true)
                        .show(ui, |ui| {
                            ui.label("Transaction type:")
                                .on_hover_text("Category of the transaction");
                            // TODO: Consider adding much more detail to the hover text. This is an
                            // important point!
                            ui.horizontal(|ui| {
                                for transaction_type in TransactionType::iter() {
                                    ui.selectable_value(
                                        &mut self.transaction_type,
                                        transaction_type.clone(),
                                        transaction_type.to_string(),
                                    );
                                }
                            });
                            ui.end_row();

                            ui.label("Transaction value:")
                                .on_hover_text("Monetary value of the transaction.");
                            ui.text_edit_singleline(&mut self.transaction_value_tentative);
                            if self.is_valid_transaction_value() {
                                ui.colored_label(
                                    Color32::from_rgb(110, 255, 110),
                                    "Valid transaction value!",
                                );
                            } else {
                                ui.colored_label(
                                    Color32::from_rgb(255, 0, 0),
                                    "Invalid transaction value!",
                                );
                            }
                            ui.end_row();

                            ui.label("Transaction currency: ")
                                .on_hover_text("Currency of the transaction.");
                            ComboBox::from_id_salt("Transaction currency")
                                .selected_text(format!("{}", self.transaction_currency))
                                .show_ui(ui, |ui| {
                                    for possible_transaction_currency in Currency::iter() {
                                        ui.selectable_value(
                                            &mut self.transaction_currency,
                                            possible_transaction_currency.clone(),
                                            format!("{possible_transaction_currency}"),
                                        );
                                    }
                                });
                            if !self.is_valid_transaction_currency()
                                & self.transaction_type.is_fund_change()
                            {
                                ui.colored_label(
                                    Color32::from_rgb(255, 0, 0),
                                    "Mismatch between transaction and account currencies!",
                                );
                            }
                            ui.end_row();

                            ui.label("Transaction date:")
                                .on_hover_text("Date in which the transaction happened.");
                            ui.add(DatePickerButton::new(&mut self.transaction_date));
                            ui.end_row();

                            if self.transaction_type.is_fund_change() {
                                ui.label("Transaction account:")
                                    .on_hover_text("Account that is affected by the transaction.");
                                ComboBox::from_id_salt("Transaction account")
                                    .selected_text(format!("{}", self.transaction_account_string))
                                    .show_ui(ui, |ui| {
                                        match self.database.iter_account_ids() {
                                            Ok(iterator) => {
                                        for account_id in iterator {
                                            if self.database.account(account_id).unwrap().currency()
                                                == &self.transaction_currency
                                            {
                                                ui.selectable_value(
                                                    &mut self.transaction_account_id,
                                                    account_id,
                                                    format!(
                                                        "{:}",
                                                        self.database
                                                            .account(account_id).unwrap()
                                                            .to_string()
                                                    ),
                                                );
                                            }
                                        }}, Err(e) => {self.throw_polars_error(e);}}
                                    });
                                if ui.button("Add new account").clicked() {
                                    self.show_input_account_window = true;
                                }
                                ui.end_row();

                                ui.label("");
                                ui.end_row();

                                ui.label("");
                                ui.end_row();

                                ui.label("");
                                ui.end_row();
                            } else {
                                // it is not fund change
                                ui.label("Transaction entity:")
                                    .on_hover_text("Entity with whom the transaction is made.");
                                containers::ComboBox::from_id_salt("Transaction entity")
                                    .selected_text(format!("{}", self.transaction_entity_string))
                                    .close_behavior(self.transaction_entity_popup)
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .text_edit_singleline(&mut self.transaction_filter)
                                            .has_focus()
                                        {
                                            self.transaction_entity_popup =
                                                PopupCloseBehavior::IgnoreClicks;
                                        } else {
                                            self.transaction_entity_popup =
                                                PopupCloseBehavior::CloseOnClick;
                                        }
                                        match self.database.iter_entity_ids() {
                                            Ok(iterator) => {
                                        for entity_id in iterator {
                                            let entity_string =
                                                self.database.entity(entity_id).unwrap().to_string();

                                            if entity_string
                                                .contains(self.transaction_filter.as_str())
                                            {
                                                ui.selectable_value(
                                                    &mut self.transaction_entity_id,
                                                    entity_id,
                                                    format!("{:}", entity_string),
                                                );
                                            }
                                        }}, Err(e) => {self.throw_polars_error(e);}}
                                    });
                                if ui.button("Add new entity").clicked() {
                                    self.show_input_entity_window = true;
                                }
                                ui.end_row();

                                ui.label("Transaction category:")
                                    .on_hover_text("Category of the transaction.");
                                match self.database.transaction_categories(&self.transaction_type) {
                                    Ok(transaction_categories) => {
                                ui.add(
                                    AutoCompleteTextEdit::new(
                                        &mut self.transaction_category,
                                    
                                            transaction_categories,
                                    )
                                    .max_suggestions(10)
                                    .highlight_matches(true),
                                );}, Err(e) => {self.throw_polars_error(e);}}
                                if self.transaction_category.len() > 0 {
                                    ui.colored_label(
                                        Color32::from_rgb(110, 255, 110),
                                        "Valid transaction category!",
                                    );
                                } else {
                                    ui.colored_label(
                                        Color32::from_rgb(255, 0, 0),
                                        "Please enter a transaction category!",
                                    );
                                }

                                ui.end_row();

                                ui.label("Transaction subcategory:")
                                    .on_hover_text("Subcategory of the transaction.");
                               match self.database.transaction_subcategories(
                                            &self.transaction_type,
                                            self.transaction_category.clone(),
                                        ) {
                                   Ok(transaction_subcategories) => {
                                ui.add(
                                    AutoCompleteTextEdit::new(
                                        &mut self.transaction_subcategory,
                                        transaction_subcategories,
                                        
                                    )
                                    .max_suggestions(10)
                                    .highlight_matches(true),
                                );}, Err(e) => {self.throw_polars_error(e);}}
                                ui.end_row();

                                ui.label("Transaction description:")
                                    .on_hover_text("Text description of the transaction.");
                                ui.text_edit_singleline(&mut self.transaction_description);
                                ui.end_row();
                            }
                        });

                    ui.separator();
                    ui.vertical_centered_justified(|ui| {
                        if self.are_valid_transaction_fields() {
                            self.transaction_value = self
                                .transaction_value_tentative
                                .parse::<f64>()
                                .expect("Error parsing transaction value");

                            let transaction: Transaction = match self.transaction_type {
                                TransactionType::Income => Transaction::Income {
                                    value: self.transaction_value,
                                    currency: self.transaction_currency.clone(),
                                    date: self.transaction_date,
                                    category: self.transaction_category.clone(),
                                    subcategory: self.transaction_subcategory.clone(),
                                    description: self.transaction_description.clone(),
                                    entity_id: self.transaction_entity_id,
                                },
                                TransactionType::Expense => Transaction::Expense {
                                    value: self.transaction_value,
                                    currency: self.transaction_currency.clone(),
                                    date: self.transaction_date,
                                    category: self.transaction_category.clone(),
                                    subcategory: self.transaction_subcategory.clone(),
                                    description: self.transaction_description.clone(),
                                    entity_id: self.transaction_entity_id,
                                },
                                TransactionType::Credit => Transaction::Credit {
                                    value: self.transaction_value,
                                    currency: self.transaction_currency.clone(),
                                    date: self.transaction_date,
                                    account_id: self.transaction_account_id,
                                },
                                TransactionType::Debit => Transaction::Debit {
                                    value: self.transaction_value,
                                    currency: self.transaction_currency.clone(),
                                    date: self.transaction_date,
                                    account_id: self.transaction_account_id,
                                },
                            };

                            if ui.button("Add transaction").clicked() {
                                self.party.add_transaction(transaction);
                                self.clear_transaction_fields();

                                self.show_input_transaction_window = false;
                            }
                        } else {
                            ui.label("Invalid transaction fields");
                        }
                    });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_input_transaction_window = false;
                }
            },
        )
    }
}
