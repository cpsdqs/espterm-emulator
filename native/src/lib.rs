#[macro_use]
extern crate neon;

pub mod terminal;

use neon::vm::{Lock};
use neon::js::{JsFunction, JsNumber, JsString, Object, JsArray};
use neon::js::class::{Class, JsClass};
use neon::mem::Handle;

use terminal::Terminal;

declare_types! {
  pub class JsTerminal for Terminal {
    init (call) {
      let scope = call.scope;

      let width = call.arguments.require(scope, 0)?.check::<JsNumber>()?.value() as u32;
      let height = call.arguments.require(scope, 1)?.check::<JsNumber>()?.value() as u32;

      Ok(Terminal::new(width, height))
    }

    method write(call) {
      let scope = call.scope;

      let data: String = call.arguments.require(scope, 0)?.check::<JsString>()?.value();
      call.arguments.this(scope).grab(|terminal| {
        terminal.write(data);
      });

      Ok(JsNumber::new(scope, 0f64).upcast())
    }

    method getCursor(call) {
      let scope = call.scope;

      let cursor = call.arguments.this(scope).grab(|terminal| { terminal.get_cursor().clone() });
      let array = JsArray::new(scope, 3);
      array.set(0, JsNumber::new(scope, cursor[0] as f64)).unwrap();
      array.set(1, JsNumber::new(scope, cursor[1] as f64)).unwrap();
      array.set(2, JsNumber::new(scope, cursor[2] as f64)).unwrap();
      Ok(JsArray::new(scope, 3).upcast())
    }

    method serializeScreen(call) {
      let scope = call.scope;

      let time = call.arguments.require(scope, 0)?.check::<JsNumber>()?.value();
      let serialized = call.arguments.this(scope).grab(|terminal| {
        terminal.serialize_screen(time).clone()
      });
      Ok(JsString::new_or_throw(scope, &serialized[..])?.upcast())
    }

    method width(call) {
      let scope = call.scope;

      let width = call.arguments.this(scope).grab(|terminal| terminal.width.clone());
      Ok(JsNumber::new(scope, width as f64).upcast())
    }

    method height(call) {
      let scope = call.scope;

      let height = call.arguments.this(scope).grab(|terminal| terminal.height.clone());
      Ok(JsNumber::new(scope, height as f64).upcast())
    }

    method getAttributes(call) {
      let scope = call.scope;

      let attributes = call.arguments.this(scope).grab(|terminal| terminal.get_attributes().clone());
      Ok(JsNumber::new(scope, attributes as f64).upcast())
    }

    method getStateID(call) {
      let scope = call.scope;

      let state_id = call.arguments.this(scope).grab(|terminal| terminal.get_state_id().clone());
      Ok(JsNumber::new(scope, state_id as f64).upcast())
    }

    method getTitle(call) {
      let scope = call.scope;

      let title = call.arguments.this(scope).grab(|terminal| terminal.get_title().clone());
      Ok(JsString::new_or_throw(scope, &title)?.upcast())
    }

    method getBellID(call) {
      let scope = call.scope;

      let bell_id = call.arguments.this(scope).grab(|terminal| terminal.get_bell_id().clone());
      Ok(JsNumber::new(scope, bell_id as f64).upcast())
    }
  }
}

register_module!(m, {
  let class: Handle<JsClass<JsTerminal>> = JsTerminal::class(m.scope)?;
  let constructor: Handle<JsFunction<JsTerminal>> = class.constructor(m.scope)?;
  m.exports.set("Terminal", constructor)?;
  Ok(())
});
