use std::{collections::HashSet, str::FromStr};

use crate::core::types::{MangaQuery, MangaSource};
use icondata::AiCaretDownOutlined;
use leptos::{prelude::*, task::spawn_local};
use leptos_meta::Title;
use leptos_use::signal_debounced;
use strum::IntoEnumIterator;
use thaw::{
    Button, ButtonAppearance, Combobox, ComboboxOption, Dialog, DialogActions, DialogBody,
    DialogContent, DialogSurface, DialogTitle, Field, Flex, FlexAlign, FlexGap, FlexJustify, Icon,
    Input, Menu, MenuItem, MenuPosition, MenuTrigger, Pagination, Spinner, SpinnerSize, Table,
    TableBody, TableCell, TableCellLayout, TableHeader, TableHeaderCell, TableRow, Toast,
    ToastBody, ToastIntent, ToastOptions, ToastTitle, ToasterInjection,
};

#[component]
pub fn Dashboard() -> impl IntoView {
    let show_add_dialog = RwSignal::new(false);
    let show_delete_dialog = RwSignal::new(false);
    let page: RwSignal<usize> = RwSignal::new(1);
    let page_count: RwSignal<usize> = RwSignal::new(1);
    let refetch_counter: RwSignal<usize> = RwSignal::new(0);

    let selected_rows = RwSignal::new(HashSet::<(MangaSource, String)>::new());
    let is_select_empty = Signal::derive(move || selected_rows.get().is_empty());

    view! {
        <Flex vertical=true gap=FlexGap::Large>
            <Title text="Dashboard" />
            <MangaTable
                current_page=page
                selected_rows=selected_rows
                refetch_counter=refetch_counter
                total_page=page_count
            />
            <Flex justify=FlexJustify::SpaceBetween>
                <Flex justify=FlexJustify::SpaceBetween gap=FlexGap::Large>
                    <Button
                        attr:id="add-btn"
                        appearance=ButtonAppearance::Primary
                        on_click=move |_| show_add_dialog.set(true)
                    >
                        "Add"
                    </Button>
                    <Button
                        attr:id="delete-btn"
                        appearance=ButtonAppearance::Primary
                        on_click=move |_| show_delete_dialog.set(true)
                        disabled=is_select_empty
                    >
                        "Delete"
                    </Button>

                </Flex>
                <Transition fallback=move || {
                    view! { <p>"Loading..."</p> }
                }>{move || view! { <Pagination page page_count /> }}</Transition>
            </Flex>
        </Flex>

        <AddMangaDialog
            open=show_add_dialog
            on_add=move || {
                refetch_counter
                    .update(|value| {
                        *value += 1;
                    });
            }
        />

        <DeleteMangaDialog
            open=show_delete_dialog
            selected_rows=selected_rows.read_only()
            on_delete=move || {
                selected_rows.update(|values| values.clear());
                refetch_counter
                    .update(|value| {
                        *value += 1;
                    });
            }
        />
    }
}

#[component]
fn MangaTable(
    current_page: RwSignal<usize>,
    selected_rows: RwSignal<HashSet<(MangaSource, String)>>,
    refetch_counter: RwSignal<usize>,
    total_page: RwSignal<usize>,
) -> impl IntoView {
    use crate::server::retrieve_manga;

    // filter
    let source_filter = RwSignal::new(None::<String>);
    let title_filter = RwSignal::new("".to_string());
    let author_filter = RwSignal::new("".to_string());
    let chapter_filter = RwSignal::new("".to_string());

    let title_filter_debounce: Signal<String> = signal_debounced(title_filter.read_only(), 250.0);
    let author_filter_debounce: Signal<String> = signal_debounced(author_filter.read_only(), 250.0);
    let chapter_filter_debounce: Signal<String> =
        signal_debounced(chapter_filter.read_only(), 250.0);

    let data_source = Resource::new(
        move || {
            (
                current_page.get(),
                source_filter.get(),
                title_filter_debounce.get(),
                author_filter_debounce.get(),
                chapter_filter_debounce.get(),
                refetch_counter.get(),
            )
        },
        move |(current_page, source, title, author, chapter_title, _counter)| async move {
            let title = match title.as_str() {
                "" => None,
                _ => Some(title),
            };

            let author = match author.as_str() {
                "" => None,
                _ => Some(author),
            };

            let chapter_title = match chapter_title.as_str() {
                "" => None,
                _ => Some(chapter_title),
            };

            retrieve_manga(
                current_page as i64,
                2,
                MangaQuery {
                    source: source.map(|s| MangaSource::from_str(&s).unwrap()),
                    title,
                    author,
                    chapter_title,
                    day: None,
                },
            )
            .await
            .unwrap()
        },
    );

    // set total pages based on datasource
    Effect::new(move |_| {
        let current_total = data_source.get().map_or(1, |d| d.total_page as usize);
        total_page.set(current_total);
    });

    view! {
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHeaderCell>"Action"</TableHeaderCell>
                    <TableHeaderCell>
                        <Menu on_select=move |_| {} position=MenuPosition::RightEnd>
                            <MenuTrigger slot>
                                <Flex align=FlexAlign::Center>
                                    <p>"Source"</p>
                                    <Icon icon=AiCaretDownOutlined width="1.5em" height="1.5em" />
                                </Flex>
                            </MenuTrigger>

                            <MenuItem value="no_icon" disabled=true>
                                <Field label="Filter Source">
                                    <Combobox
                                        selected_options=source_filter
                                        placeholder="Select a source"
                                        clearable=true
                                    >
                                        {move || {
                                            MangaSource::iter()
                                                .map(|s| {
                                                    view! {
                                                        <ComboboxOption value=s.to_string() text=s.to_string() />
                                                    }
                                                })
                                                .collect_view()
                                        }}

                                    </Combobox>
                                </Field>
                            </MenuItem>
                        </Menu>
                    </TableHeaderCell>
                    <TableHeaderCell>
                        <Menu on_select=move |_| {} position=MenuPosition::RightEnd>
                            <MenuTrigger slot>
                                <Flex align=FlexAlign::Center>
                                    <p>"Title"</p>
                                    <Icon icon=AiCaretDownOutlined width="1.5em" height="1.5em" />
                                </Flex>
                            </MenuTrigger>

                            <MenuItem value="no_icon" disabled=true>
                                <Field label="Filter Title">
                                    <Input value=title_filter />
                                </Field>
                            </MenuItem>
                        </Menu>
                    </TableHeaderCell>
                    <TableHeaderCell>
                        <Menu on_select=move |_| {} position=MenuPosition::RightEnd>
                            <MenuTrigger slot>
                                <Flex align=FlexAlign::Center>
                                    <p>"Author"</p>
                                    <Icon icon=AiCaretDownOutlined width="1.5em" height="1.5em" />
                                </Flex>
                            </MenuTrigger>

                            <MenuItem value="no_icon" disabled=true>
                                <Field label="Filter Author">
                                    <Input value=author_filter />
                                </Field>
                            </MenuItem>
                        </Menu>
                    </TableHeaderCell>
                    <TableHeaderCell>
                        <Menu on_select=move |_| {} position=MenuPosition::RightEnd>
                            <MenuTrigger slot>
                                <Flex align=FlexAlign::Center>
                                    <p>"Chapter Name"</p>
                                    <Icon icon=AiCaretDownOutlined width="1.5em" height="1.5em" />
                                </Flex>
                            </MenuTrigger>

                            <MenuItem value="no_icon" disabled=true>
                                <Field label="Filter Chapter">
                                    <Input value=chapter_filter />
                                </Field>
                            </MenuItem>
                        </Menu>
                    </TableHeaderCell>
                </TableRow>
            </TableHeader>
            <TableBody>
                <Transition fallback=move || {
                    view! {
                        <TableRow>
                            <p>"Loading..."</p>
                        </TableRow>
                    }
                }>
                    {move || Suspend::new(async move {
                        data_source
                            .await
                            .data
                            .into_iter()
                            .map(|(source, manga_id, manga)| {
                                let src = source.clone();
                                let src_check = source.clone();
                                let id_check = manga_id.clone();
                                view! {
                                    <TableRow>
                                        <TableCell>
                                            <input
                                                type="checkbox"
                                                style="transform: scale(1.3)"
                                                on:change:target=move |ev| {
                                                    match ev.target().checked() {
                                                        true => {
                                                            selected_rows
                                                                .update(|value| {
                                                                    value.insert((src.clone(), manga_id.clone()));
                                                                })
                                                        }
                                                        false => {
                                                            selected_rows
                                                                .update(|value| {
                                                                    value.remove(&(src.clone(), manga_id.clone()));
                                                                })
                                                        }
                                                    };
                                                }
                                                prop:checked=move || {
                                                    selected_rows
                                                        .get()
                                                        .contains(&(src_check.clone(), id_check.clone()))
                                                }
                                            />
                                        </TableCell>
                                        <TableCell>
                                            <TableCellLayout>{source.to_string()}</TableCellLayout>
                                        </TableCell>
                                        <TableCell>
                                            <TableCellLayout>{manga.title}</TableCellLayout>
                                        </TableCell>
                                        <TableCell>
                                            <TableCellLayout>{manga.author}</TableCellLayout>
                                        </TableCell>
                                        <TableCell>
                                            <TableCellLayout>
                                                {manga.latest_chapter_title}
                                            </TableCellLayout>
                                        </TableCell>
                                    </TableRow>
                                }
                            })
                            .collect_view()
                    })}
                </Transition>
            </TableBody>
        </Table>
    }
}

#[component]
fn AddMangaDialog(open: RwSignal<bool>, #[prop(into)] on_add: Callback<()>) -> impl IntoView {
    use crate::server::add_manga;

    // state
    let manga_id = RwSignal::new("".to_owned());
    let selected_source = RwSignal::new(None::<String>);
    let is_submitting = RwSignal::new(false);

    let toaster = ToasterInjection::expect_context();
    let handle_add = move |_| {
        let id = manga_id.get();
        let source = selected_source
            .get()
            .map(|s| MangaSource::from_str(&s).unwrap());

        spawn_local(async move {
            is_submitting.set(true);
            let result = add_manga(id, source).await;

            match result {
                Ok(manga) => {
                    toaster.dispatch_toast(
                        move || view! {
                            <Toast>
                                <ToastTitle>"Add Success"</ToastTitle>
                                <ToastBody>{format!("Success adding {}", manga.title)}</ToastBody>
                            </Toast>
                        },
                        ToastOptions::default().with_intent(ToastIntent::Success),
                    );

                    manga_id.set("".into());
                    selected_source.set(None);
                    open.set(false);
                    on_add.run(());
                }
                Err(e) => toaster.dispatch_toast(
                    move || {
                        view! {
                            <Toast>
                                <ToastTitle>"Error"</ToastTitle>
                                <ToastBody>{e.to_string()}</ToastBody>
                            </Toast>
                        }
                    },
                    ToastOptions::default().with_intent(ToastIntent::Error),
                ),
            }

            is_submitting.set(false);
        })
    };

    view! {
        <Dialog open>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>"Add new Manga"</DialogTitle>
                    <DialogContent>
                        <Flex vertical=true gap=FlexGap::Large style="margin-bottom: 10px">
                            <Field label="Manga ID">
                                <Input value=manga_id />
                            </Field>

                            <Field label="Source">
                                <Combobox
                                    selected_options=selected_source
                                    placeholder="Select a source"
                                >
                                    {move || {
                                        MangaSource::iter()
                                            .map(|s| {
                                                view! {
                                                    <ComboboxOption value=s.to_string() text=s.to_string() />
                                                }
                                            })
                                            .collect_view()
                                    }}

                                </Combobox>
                            </Field>
                        </Flex>
                    </DialogContent>

                    <DialogActions>
                        <Button
                            appearance=ButtonAppearance::Primary
                            on_click=handle_add
                            disabled=is_submitting
                        >
                            {move || {
                                is_submitting
                                    .get()
                                    .then(|| view! { <Spinner size=SpinnerSize::Tiny /> })
                            }}
                            "Add"
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
}

#[component]
fn DeleteMangaDialog(
    open: RwSignal<bool>,
    selected_rows: ReadSignal<HashSet<(MangaSource, String)>>,
    #[prop(into)] on_delete: Callback<()>,
) -> impl IntoView {
    use crate::server::delete_manga;

    // state
    let is_submitting = RwSignal::new(false);

    let toaster = ToasterInjection::expect_context();
    let handle_delete =
        move |_| {
            spawn_local(async move {
                is_submitting.set(true);
                let values = selected_rows
                    .get_untracked()
                    .into_iter()
                    .collect::<Vec<_>>();
                let result = delete_manga(values).await;

                match result {
                    Ok(num_rows) => {
                        toaster.dispatch_toast(
                        move || view! {
                            <Toast>
                                <ToastTitle>"Delete Success"</ToastTitle>
                                <ToastBody>{format!("{} manga deleted", num_rows)}</ToastBody>
                            </Toast>
                        },
                        ToastOptions::default().with_intent(ToastIntent::Success),
                    );
                        on_delete.run(());
                    }
                    Err(e) => toaster.dispatch_toast(
                        move || {
                            view! {
                                <Toast>
                                    <ToastTitle>"Error"</ToastTitle>
                                    <ToastBody>{e.to_string()}</ToastBody>
                                </Toast>
                            }
                        },
                        ToastOptions::default().with_intent(ToastIntent::Error),
                    ),
                }

                is_submitting.set(false);
                open.set(false);
            })
        };

    view! {
        <Dialog open>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>"Delete Manga"</DialogTitle>
                    <DialogContent>
                        <p>"Are you sure to delete selected manga ?"</p>
                    </DialogContent>

                    <DialogActions>
                        <Button
                            appearance=ButtonAppearance::Primary
                            on_click=handle_delete
                            disabled=is_submitting
                        >
                            {move || {
                                is_submitting
                                    .get()
                                    .then(|| view! { <Spinner size=SpinnerSize::Tiny /> })
                            }}
                            "Yes"
                        </Button>
                        <Button
                            appearance=ButtonAppearance::Primary
                            on_click=move |_| open.set(false)
                        >
                            "No"
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
}
