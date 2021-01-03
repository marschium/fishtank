#[derive(new)]
pub(crate) struct DebugInfo {
    #[new(value = "None")]
    pub world_pos: Option::<(usize, usize)>,
    #[new(value = "false")]
    pub spawning: bool
}