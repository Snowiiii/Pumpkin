pub trait Command<'a> {
    const NAME: &'a str;
    const ALIEASES: [&'a str];
    const DESCRIPTION: &'a str;
    
    fn on_execute(client: &mut Cli);
}