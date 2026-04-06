pub struct Session {
    pub id: Option<String>,
    pub model: Option<String>,
    pub tools: Vec<String>,
    pub slash_commands: Vec<String>,
    pub skills: Vec<String>,
    pub plugins: Vec<String>,
    pub agents: Vec<String>,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub turns: u32,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: None,
            model: None,
            tools: Vec::new(),
            slash_commands: Vec::new(),
            skills: Vec::new(),
            plugins: Vec::new(),
            agents: Vec::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cost_usd: 0.0,
            turns: 0,
        }
    }

    pub fn update_from_init(
        &mut self,
        session_id: &str,
        model: Option<&str>,
        tools: Vec<String>,
        slash_commands: Vec<String>,
        skills: Vec<String>,
        plugins: Vec<String>,
        agents: Vec<String>,
    ) {
        self.id = Some(session_id.to_string());
        if let Some(m) = model {
            self.model = Some(m.to_string());
        }
        self.tools = tools;
        self.slash_commands = slash_commands;
        self.skills = skills;
        self.plugins = plugins;
        self.agents = agents;
    }

    pub fn track_result(&mut self, turns: u32, cost: f64, input_tokens: u64, output_tokens: u64) {
        self.turns += turns;
        self.total_cost_usd += cost;
        self.total_input_tokens += input_tokens;
        self.total_output_tokens += output_tokens;
    }
}
