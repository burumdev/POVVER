use std::{
    sync::Arc,
    thread,
};
use tokio::sync::broadcast as tokio_broadcast;

use slint::{ModelRc, CloseRequestResponse, SharedString, Model, VecModel, FilterModel, ToSharedString};

use crate::{
    app_state::StatePayload,
    simulation::{
        SimInt,
        StateAction,
        timer::TimerEvent,
        EconUpdate,
        hub_comms::MessageEntity,
    },
    logger::LogMessage as LoggerMessage,
};

pub enum UIFlag {
    Pause,
    Quit,
    SpeedChange(SimInt),
}

slint::include_modules!();

pub struct UIController {}

impl UIController {
    pub fn new() -> Self {
        Self {}
    }
}

impl UIController {
    pub fn run(
        &self,
        flag_sender: crossbeam_channel::Sender<UIFlag>,
        mut wakeup_receiver: tokio_broadcast::Receiver<StateAction>,
        mut log_receiver: tokio_broadcast::Receiver<LoggerMessage>,
        state: Arc<StatePayload>,
        factory_count: usize,
    ) -> thread::JoinHandle<()> {
        let flag_sender_close = flag_sender.clone();
        let flag_sender_speed = flag_sender.clone();

        thread::Builder::new().name("POVVER_UI".to_string()).spawn(move || {
            let app = PovverMain::new().unwrap();

            // Event handlers
            app.global::<GlobCallbacks>().on_toggle_pause(move || {
                flag_sender.send(UIFlag::Pause).unwrap();
            });
            app.global::<GlobCallbacks>().on_speed_change(move |speed_index| {
                flag_sender_speed.send(UIFlag::SpeedChange(speed_index)).unwrap();
            });
            app.window().on_close_requested(move || {
                println!("UI: Shutting down the user interface");
                flag_sender_close.send(UIFlag::Quit).unwrap();

                CloseRequestResponse::HideWindow
            });

            // UI state updates
            let app_weak = app.as_weak();
            slint::spawn_local(async move {
                let appw = app_weak.clone().unwrap();

                // Start the UI maximized to the screen it's launched on
                appw.window().set_maximized(true);

                // Initial log messages setup
                appw.set_messages(ModelRc::from(VecModel::from_slice(&[])));
                let messages_rc = appw.get_messages();
                let messages_model = messages_rc.as_any().downcast_ref::<VecModel<LogMessage>>().unwrap();

                let hub_filtered = FilterModel::from(messages_rc.clone().filter(|msg| msg.source == MessageSource::Hub));
                let pp_filtered = FilterModel::from(messages_rc.clone().filter(|msg| msg.source == MessageSource::PP));

                let factory_ids = (0..factory_count).map(|i| i as SimInt).collect::<Vec<SimInt>>();
                let factory_model_empty = factory_ids.iter().map(|_| ModelRc::from(VecModel::from_slice(&[]))).collect::<Vec<_>>();
                let factory_filtered: ModelRc<ModelRc<LogMessage>> = ModelRc::from(VecModel::from_slice(&factory_model_empty));

                // This is necessary gymnastics in order to prevent rerender of factories on the map
                // Every time factory states change. We use a simple integer that doesn't change
                // to prevent the rerenders.
                appw.set_factory_count(factory_count as SimInt);

                // Main UI loop. This will update UI state when signals from the outside pour in
                while let Ok(action) = wakeup_receiver.recv().await {
                    match action {
                        StateAction::Timer(event) => {
                            {
                                let timer_lock = state.timer.read().unwrap();
                                appw.set_timer(
                                    TimerData {
                                        date: timer_lock.date.clone(),
                                    }
                                );
                            }

                            if event.at_least_month() {
                                {
                                    let timer_lock = state.timer.read().unwrap();
                                    appw.set_month(
                                        MonthData {
                                            day_start: timer_lock.month_data.day_start,
                                            day_end: timer_lock.month_data.day_end,
                                            name: SharedString::from(timer_lock.month_data.name),
                                            sunshine_factor: timer_lock.month_data.sunshine_factor,
                                            windspeed_factor: timer_lock.month_data.windspeed_factor,
                                            cloud_forming_factor: timer_lock.month_data.cloud_forming_factor,
                                        }
                                    );
                                }
                            };

                            match event {
                                TimerEvent::NothingUnusual => {
                                    {
                                        let timer_lock = state.timer.read().unwrap();
                                        if timer_lock.date.minute % 4 == 0 {
                                            let pp_lock = state.povver_plant.read().unwrap();
                                            appw.set_pp(PPState {
                                                fuel: pp_lock.fuel,
                                                fuel_capacity: pp_lock.fuel_capacity,
                                                balance: pp_lock.balance.val(),
                                                is_awaiting_fuel: pp_lock.is_awaiting_fuel,
                                                is_awaiting_fuel_capacity: pp_lock.is_awaiting_fuel_capacity,
                                                is_awaiting_production_capacity: pp_lock.is_awaiting_production_capacity,
                                                is_bankrupt: pp_lock.is_bankrupt,
                                                production_capacity: pp_lock.production_capacity.val(),
                                            })
                                        }
                                        if timer_lock.date.minute % 7 == 0 {
                                            let factories_lock = state.factories.read().unwrap();
                                            appw.set_factories(
                                                ModelRc::from(
                                                    factories_lock.iter().map(|fs| {
                                                        let fstate = fs.read().unwrap();
                                                        let product_stocks = ModelRc::from(fstate.product_stocks.iter().map(|ps| ProductStock {
                                                            name: ps.product.name.to_shared_string(),
                                                            amount: ps.units
                                                        }).collect::<Vec<ProductStock>>().as_slice());
                                                        let product_portfolio = ModelRc::from(fstate.product_portfolio.iter().map(|port|
                                                            port.name.to_shared_string()).collect::<Vec<SharedString>>().as_slice()
                                                        );

                                                        FactoryState {
                                                            id: fstate.id as SimInt,
                                                            balance: fstate.balance.val(),
                                                            available_energy: fstate.available_energy.val(),
                                                            product_stocks,
                                                            solarpanels: fstate.solarpanels.len() as SimInt,
                                                            industry: fstate.industry.name.to_shared_string(),
                                                            product_portfolio,
                                                            is_bankrupt: fstate.is_bankrupt,
                                                            is_awaiting_solarpanels: fstate.is_awaiting_solarpanels,
                                                        }
                                                    }).collect::<Vec<FactoryState>>().as_slice(),
                                                )
                                            )
                                        }
                                    }
                                    if let Ok(message) = log_receiver.try_recv() {
                                        // I'm not sure if it's the most efficient way to do this categorization of
                                        // Factory messages based on factory id (index) and dynamic multidimensional array.
                                        // But it seems to work fine. We had to do this gymnastics because either:
                                        // 1. We don't have sufficient skill with the Slint internal vector types and transforming to them
                                        // 2. Slint has no support for a) Rust types as native types and b) thread safe types so this mess is inevitable.
                                        if let MessageEntity::Factory(fac_id) = message.source {
                                            let row_data = factory_filtered.row_data(fac_id as usize);
                                            if let Some(mrc) = row_data {
                                                let dcast = mrc.as_any().downcast_ref::<VecModel<LogMessage>>();
                                                if let Some(vec) = dcast {
                                                    vec.push(message.clone().into());
                                                }
                                            } else {
                                                factory_filtered.set_row_data(fac_id as usize, ModelRc::from(VecModel::from_slice(&[message.clone().into()])));
                                            }
                                        }

                                        if messages_model.iter().len() >= 20 {
                                            messages_model.remove(0);
                                        }
                                        messages_model.push(message.into());

                                        appw.set_category_messages({
                                            CategoryMessages {
                                                hub: ModelRc::new(VecModel::from_iter(hub_filtered.iter())),
                                                pp: ModelRc::new(VecModel::from_iter(pp_filtered.iter())),
                                                // We've got to clone every time we assign the factory messages
                                                // But this should be a cheap Rc clone.
                                                factory: factory_filtered.clone(),
                                            }
                                        });
                                    }
                                },
                                _ => ()
                            }
                        },
                        StateAction::Env => {
                            let env_lock = state.env.read().unwrap();
                            appw.set_env(
                                EnvData {
                                    the_sun: (&env_lock.the_sun).into(),
                                    wind_direction: env_lock.wind_direction,
                                    wind_speed: env_lock.wind_speed.val(),
                                    wind_speed_level: WindSpeedLevel::from(&env_lock.wind_speed),
                                    clouds: ModelRc::from(env_lock.clouds.as_slice()),
                                }
                            );
                        },
                        StateAction::EconUpdate(update_type) => {
                            match update_type {
                                EconUpdate::Macro => {
                                    let econ_lock = state.economy.read().unwrap();
                                    appw.set_macroecon(
                                        MacroEconData {
                                            fuel_price: econ_lock.fuel_price.val(),
                                            inflation_direction: econ_lock.inflation_direction.clone().into(),
                                            inflation_rate: econ_lock.inflation_rate,
                                        }
                                    )
                                },
                                EconUpdate::Demands => {
                                    let econ_lock = state.economy.read().unwrap();
                                    appw.set_product_demands(
                                        ModelRc::from(
                                            econ_lock.product_demands
                                                .iter()
                                                .map(|demand| demand.into())
                                                .collect::<Vec<ProductDemand>>()
                                                .as_slice()
                                        )
                                    );
                                }
                            }
                        }
                        StateAction::Misc => {
                            let misc_lock = state.misc.lock().unwrap();
                            appw.set_misc(UIMisc {
                                is_paused: misc_lock.is_paused,
                                speed_index: misc_lock.speed_index as SimInt,
                            });
                        },
                        StateAction::Quit => {
                            break;
                        },
                        _ => ()
                    }
                }
            }).unwrap();

            // Run the UI
            app.run().unwrap();
        }).unwrap()
    }
}
