import {applyMiddleware, createStore} from 'redux'
//import {decode} from '../util/probor'
import {CANCEL} from 'khufu-runtime'

export const ERROR = '@@request/error'
export const DATA = '@@request/data'
export const UPDATE_REQUEST = '@@request/update_request'

const DEBOUNCE_DELAY = 50


export function data(state=null, action) {
    switch(action.type) {
        case DATA:
            return action.data;
    }
    return state;
}

export var pick = key => (state=null, action) => {
    switch(action.type) {
        case DATA:
            return action.data[key];
    }
    return state;
}

export var refresher = store => next => {
    var url
    var delay = 5000
    var body
    var response_type = 'json'
    var headers = {}
    var decoder = x => x
    var timeout
    var request
    var updated

    function start() {
        if(timeout) {
            clearTimeout(timeout)
            timeout = null
        }
        updated = false;

        request = new XMLHttpRequest();
        var time = new Date();
        request.responseType = response_type
        request.onreadystatechange = (ev) => {
            if(request.readyState < 4) {
                return;
            }
            var lcy = new Date() - time;
            let req  = request;

            request = null; // not processing any more
            timeout = setTimeout(start, updated ? DEBOUNCE_DELAY : delay)

            if(req.status != 200 && req.status != 206) {
                next({type: ERROR, request: req, latency: lcy})
                return;
            }
            try {
                next({
                    type: DATA,
                    data: decoder(req.response),
                    req: req,
                    latency: lcy,
                })
            } catch(e) {
                next({type: ERROR, exception: e, latency: lcy})
            }
        }
        if(body) {
            request.open('POST', url, true);
            request.setRequestHeader('Content-Type', 'application/json')
            for(var i in headers) {
                request.setRequestHeader(i, headers[i]);
            }
            request.send(body)
        } else {
            request.open('GET', url, true);
            for(var i in headers) {
                request.setRequestHeader(i, headers[i]);
            }
            request.send()
        }
    }
    function stop() {
        if(request) {
            request.onreadystatechange = null
            request.abort()
            request = null
        }
        if(timeout) {
            clearTimeout(timeout)
            timeout = null
        }
    }

    return action => {
        switch(action.type) {
            case UPDATE_REQUEST:
                url = action.url || url
                headers = action.headers || {}
                delay = action.delay || delay
                body = action.body || body
                response_type = action.response_type || response_type
                decoder = action.decoder || decoder
                if(!request && !timeout) {
                    // initialized
                    start()
                } else if(action.immediate) {
                    if(request) {
                        updated = true
                    } else {
                        if(timeout) clearTimeout(timeout)
                        setTimeout(start, DEBOUNCE_DELAY)
                    }
                }
                break;
            case CANCEL:
                stop()
                break;
            default:
                break;
        }
        return next(action)
    }
}

export var json = url => ({
    type: UPDATE_REQUEST,
    url: url,
    response_type: 'json',
    decoder: x => x,
})

export var fast_refresh_json = url => ({
    type: UPDATE_REQUEST,
    url: url,
    response_type: 'json',
    decoder: x => x,
    delay: 500,
})

export var plain = url => ({
    type: UPDATE_REQUEST,
    url: url,
    response_type: 'text',
    decoder: x => x,
})
