#[macro_use]
extern crate neon;

pub mod terminal;

use neon::vm::{Lock};
use neon::js::{JsFunction, JsNumber, JsString, Object};
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

    method serialize(call) {
      let scope = call.scope;

      let serialized = call.arguments.this(scope).grab(|terminal| {
        terminal.serialize().clone()
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
  }
}

register_module!(m, {
  let class: Handle<JsClass<JsTerminal>> = JsTerminal::class(m.scope)?;
  let constructor: Handle<JsFunction<JsTerminal>> = class.constructor(m.scope)?;
  m.exports.set("Terminal", constructor)?;
  Ok(())
});
