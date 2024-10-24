pub async fn update_books() {
    // retrieve series from db (paginated) based on the current day
    // for each series check for latest update

    // not released -> released and title doesn't change -> mark as released
    // released -> not released and title change -> update metadata and mark as upcoming
    // released -> released and title change -> mark as released and update metadata
    todo!()
}
