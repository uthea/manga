use std::str::FromStr;

use crate::core::types::{MangaQuery, MangaSource};
use leptos::{prelude::*, task::spawn_local};
use leptos_meta::Title;
use strum::IntoEnumIterator;
use thaw::{
    Button, ButtonAppearance, Combobox, ComboboxOption, Dialog, DialogActions, DialogBody,
    DialogContent, DialogSurface, DialogTitle, Field, Flex, FlexGap, FlexJustify, Input,
    Pagination, Spinner, SpinnerSize, Table, TableBody, TableCell, TableCellLayout, TableHeader,
    TableHeaderCell, TableRow, Toast, ToastBody, ToastIntent, ToastOptions, ToastTitle,
    ToasterInjection,
};

#[component]
pub fn Dashboard() -> impl IntoView {
    use crate::server::retrieve_manga;

    let show_add_dialog = RwSignal::new(false);
    let page: RwSignal<usize> = RwSignal::new(1);
    let query_option = RwSignal::new(MangaQuery {
        source: None,
        day: None,
    });

    let data_source = Resource::new(
        move || (page.get(), query_option.get()),
        move |(current_page, query_op)| async move {
            retrieve_manga(current_page as i64, 10, query_op)
                .await
                .unwrap()
        },
    );

    let page_count = Signal::derive(move || data_source.get().map_or(1, |d| d.total_page as usize));

    view! {
        <Flex vertical=true gap=FlexGap::Large>
            <Title text="Dashboard" />
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHeaderCell>"Source"</TableHeaderCell>
                        <TableHeaderCell>"Title"</TableHeaderCell>
                        <TableHeaderCell>"Author"</TableHeaderCell>
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
                        {move || {
                            data_source
                                .get()
                                .map(|ds| {
                                    ds.data
                                        .into_iter()
                                        .map(|(source, manga)| {
                                            view! {
                                                <TableRow>
                                                    <TableCell>
                                                        <TableCellLayout>{source.to_string()}</TableCellLayout>
                                                    </TableCell>
                                                    <TableCell>
                                                        <TableCellLayout>{manga.title}</TableCellLayout>
                                                    </TableCell>
                                                    <TableCell>
                                                        <TableCellLayout>{manga.author}</TableCellLayout>
                                                    </TableCell>
                                                </TableRow>
                                            }
                                        })
                                        .collect_view()
                                })
                        }}
                    </Transition>
                </TableBody>
            </Table>
            <Flex justify=FlexJustify::SpaceBetween>
                <Button
                    appearance=ButtonAppearance::Primary
                    on_click=move |_| show_add_dialog.set(true)
                >
                    "Add"
                </Button>
                <Transition fallback=move || {
                    view! { <p>"Loading..."</p> }
                }>{move || view! { <Pagination page page_count /> }}</Transition>
            </Flex>
        </Flex>

        <AddMangaDialog
            open=show_add_dialog
            on_add=move || {
                data_source.refetch();
            }
        />
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
    let on_add = move |_| {
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
                            on_click=on_add
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
