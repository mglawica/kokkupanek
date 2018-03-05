export var graphql_action = (query, extra_vars, form_var) => store => next => action => {
    switch(action.type) {
         case 'submit':
            let data
            if(form_var) {
                data = {[form_var]: store.getState(), ...extra_vars}
            } else {
                data = {...store.getState(), ...extra_vars}
            }
            store.dispatch({type: 'loading'})
            fetch('/v1/wait_action', {
                    method: 'POST',
                    body: JSON.stringify({
                        'query': query,
                        'variables': data,
                    }),
                })
                .then(req => req.json())
                .then(result => {
                    if(!result.errors || result.errors.length == 0) {
                        store.dispatch({
                            type: 'success',
                            data: result.data,
                            form_data: data,
                        })
                    } else {
                        store.dispatch({type: 'error', errors: result.errors})
                    }
                })
                .catch(e => {
                    store.dispatch({type: 'error', error: data})
                })
            break;
    }
    return next(action)
}

export var on_success = (sstore, func, ...args) => store => next => action => {
    switch(action.type) {
        case 'success':
            sstore.dispatch(func(...args))
            break
    }
    return next(action)
}
