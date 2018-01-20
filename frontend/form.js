import {CANCEL} from 'khufu-runtime'


let GLOBAL_COUNTER = 0


export class Form {
    constructor() {
        this._fields = {}
        this._listeners = []
    }
    field(name, default_value) {
        return new Field(this, name, default_value)
    }
    getState() {
        let r = {}
        for(var k in this._fields) {
            r[k] = this._fields[k].getState()
        }
        console.log("FIELDS", this._fields, r)
        return r
    }
    dispatch(action) {
        switch(action.type) {
            case CANCEL:
                break;
        }
    }
    subscribe(callback) {
        console.error("SUBFORM")
        this._listeners.push(callback)
        return function() {
            let idx = this._listeners.indexOf(callback);
            if(idx >= 0) {
                this._listeners.splice(idx, 1);
            }
        }
    }
    _add_field(name, field) {
        this._fields[name] = field
    }
    _remove_field(name, field) {
        if(this._fields[name] == field) {
            delete this._fields
        }
    }
    _trigger() {
        for(let list of this._listeners) {
            list()
        }
    }
}

export class Field {
    constructor(form, name, default_value) {
        this._name = name
        this._form = form
        this._value = default_value
        this.id = 'field_' + (GLOBAL_COUNTER += 1)
        form._add_field(name, this)
    }
    getState() {
        return this._value
    }
    subscribe(callback) {
        console.log("SUBFIELD")
        return this._form.subscribe(callback)
    }
    dispatch(action) {
        switch(action.type) {
            case 'set':
                this._value = action.value
                this._form._trigger()
                break;
            case CANCEL:
                form._remove_field(this._name, this)
                break;
        }
    }
}

export function form() {
    return new Form()
}
