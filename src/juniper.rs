pub fn execute_action(action: &GraphqlAction, schedule: &RefCell<Schedule>)
    -> Value
{
    let result = execute(&action.query,
        action.operation_name.as_ref().map(|x| &x[..]),
        &Schema::new(&Query, &Mutation),
        &action.variables,
        &Context { schedule },
    );
    match result {
        Ok((data, errors)) => {
            json!({"data": data, "errors": errors})
        }
        Err(err) => {
            to_value(&err).expect("can serialize juniper's error")
        }
    }
}
