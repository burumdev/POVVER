use std::{
    sync::Arc,
    thread,
};

use tokio::sync::broadcast as tokio_broadcast;

use slint::{ModelRc, CloseRequestResponse, SharedString, Model, VecModel, FilterModel};

use crate::{
    app_state::StatePayload,
    simulation::{SimInt, StateAction, timer::TimerEvent},
    logger::LogMessage as LoggerMessage,
};
use crate::simulation::EconUpdate;

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
    ) -> thread::JoinHandle<()> {
        let flag_sender_close = flag_sender.clone();
        let flag_sender_speed = flag_sender.clone();

        thread::Builder::new().name("POVVER_UI".to_string()).spawn(move || {
            let app = PovverMain::new().unwrap();

            // Event handlers
            app.on_toggle_pause(move || {
                flag_sender.send(UIFlag::Pause).unwrap();
            });
            app.on_speed_change(move |speed_index| {
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
                appw.window().set_maximized(true);

                appw.set_messages(ModelRc::from(VecModel::from_slice(&[])));
                let messages_rc = appw.get_messages();
                let messages_model = messages_rc.as_any().downcast_ref::<VecModel<LogMessage>>().unwrap();

                let hub_filtered = FilterModel::from(messages_rc.clone().filter(|msg| msg.source == MessageSource::Hub));
                let pp_filtered = FilterModel::from(messages_rc.clone().filter(|msg| msg.source == MessageSource::PP));
                let factory_filtered = FilterModel::from(messages_rc.clone().filter(|msg| msg.source == MessageSource::Factory));

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

                            match event {
                                te if te.at_least_month() => {
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
                                },
                                TimerEvent::NothingUnusual => {
                                    if let Ok(message) = log_receiver.try_recv() {
                                        if messages_model.iter().len() >= 50 {
                                            messages_model.remove(0);
                                        }
                                        messages_model.push(message.into());

                                        appw.set_category_messages({
                                            CategoryMessages {
                                                hub: ModelRc::new(VecModel::from_iter(hub_filtered.iter())),
                                                pp: ModelRc::new(VecModel::from_iter(pp_filtered.iter())),
                                                factory: ModelRc::new(VecModel::from_iter(factory_filtered.iter())),
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
