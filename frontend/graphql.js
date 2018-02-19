export var graphql_action = (query, extra_vars, form_var) => store => next => action => {
    switch(action.type) {
         case 'submit':
            let data
            if(form_var) {
                data = {[form_var]: store.getState(), ...extra_vars}
            } else {
                data = {...store.getState(), ...extra_vars}
            }
            fetch('/v1/action', {
                method: 'POST',
                body: JSON.stringify({
                    'query': query,
                    'variables': data,
                }),
            })
            break;
    }
    return next(action)
}
