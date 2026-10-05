/// ---------------------------------- utils ----------------------------------

#let has-keys(dict, keys) = {
  if type(dict) != dictionary or dict.keys().len() != keys.len() {
    return false
  }

  for k in keys {
    if k not in dict {
      return false
    }
  }

  return true
}

#let has-type(x, types) = { x in types or type(x) in types }

#let assert-type(x, types) = {
  assert(
    has-type(x, types),
    message: (
      str(type(x)),
      "does not have types:",
      repr(types),
    ).join(" "),
  )
}

/// ---------------------------------- units ----------------------------------

#let new-unit(
  numerator: (:),
  denominator: (:),

  currency: none,
) = {
  if currency != none {
    let currency = currency.split("/")
    let numerator = (currency.at(0): 1)
    let denominator = if currency.len() == 2 { (currency.at(1): 1) } else { (:) }
    return (numerator: numerator, denominator: denominator)
  }

  return (numerator: numerator, denominator: denominator)

  panic("unit: invalid args")
}

#let is-unit(x) = { type(x) == dictionary and has-keys(x, ("numerator", "denominator")) }

#let scalar() = new-unit(numerator: (:), denominator: (:))

#let unit-all-eq(units) = {
  let first = units.at(0)
  units.all(u => u == first)
}

#let show-unit(unit) = {
  if unit == none {
    return
  }

  assert(has-keys(unit, ("numerator", "denominator")))

  let show-multiset(m) = {
    m.pairs().map(((u, m)) => [#u#if m > 1 { super([#m]) }]).join(sym.dot.c)
  }

  if unit.numerator == (:) and unit.denominator == (:) {
    return []
  }

  if unit.denominator == (:) {
    return show-multiset(unit.numerator)
  }

  math.frac(show-multiset(unit.numerator), show-multiset(unit.denominator))
}

#let multiset-update(a, b) = {
  // a way to clone?
  let new = b + (:)
  for (k, v) in a.pairs() {
    let v = v + b.at(k, default: 0)
    new.insert(k, v)
  }

  new
}

#let unit-mul(a, b) = {
  let n = multiset-update(a.numerator, b.numerator)
  let d = multiset-update(a.denominator, b.denominator)

  for (u, nn) in n.pairs() {
    let nd = d.at(u, default: none)
    if nd == none {
      continue
    }

    if nn == nd {
      let _ = n.remove(u)
      let _ = d.remove(u)
    }
    if nn < nd {
      let _ = n.remove(u)
      let _ = d.insert(u, nd - nn)
    }
    if nn > nd {
      let _ = d.remove(u)
      let _ = n.insert(u, nn - nd)
    }
  }

  new-unit(numerator: n, denominator: d)
}


/// ---------------------------------- forms ----------------------------------

#let is-numeric(string) = { string.matches(regex("^-?\d+\.\d\d$")) }

#let new-value(
  // parse decimal value and unit
  parse: none,

  // explicit value and unit
  value: none,
  unit: none,

  // can always be provided
  desc: none,
) = {
  let (value, unit) = if parse != none {
    let parts = parse.split(" ")
    let value = parts.at(0)
    let unit = if parts.len() > 1 { new-unit(currency: parts.at(1)) } else { scalar() }

    (decimal(value), unit)
  } else if value != none {
    assert-type(value, (decimal, str))
    assert-type(unit, (dictionary, none))

    (value, unit)
  } else {
    panic(
      "invalid value",
      (
        value: value,
        unit: value,
        desc: value,
        parse: value,
        string: value,
      ),
    )
  }

  (value: value, unit: unit, desc: desc)
}

#let is-value(value) = {
  type(value) == dictionary and has-keys(value, ("value", "unit", "desc"))
}

#let sum(..fields) = (
  op: "+",
  val-fn: (..values) => { values.pos().sum() },
  unit-fn: (..units) => {
    if unit-all-eq(units.pos()) {
      units.pos().at(0)
    } else {
      panic("sum: units not equal: " + repr(units.pos()))
    }
  },
  args: fields.pos(),
)
#let mul(a, b) = (
  op: "*",
  val-fn: (a, b) => { a * b },
  unit-fn: unit-mul,
  args: (a, b),
)
#let dif(a, b) = (
  op: "-",
  val-fn: (a, b) => { a - b },
  unit-fn: (a, b) => {
    if a == b {
      a
    } else {
      panic("dif: units not equal: " + repr((a, b)))
    }
  },
  args: (a, b),
)

#let to-tables(docs: (:), forms: (:)) = {
  let follow(desc, default: none) = {
    if type(desc) == str {
      return desc
    }

    if "inner" not in desc {
      return default
    }

    follow(desc.inner, default: desc.ref)
  }

  let show-desc(desc) = {
    if type(desc) == str {
       desc
    } else if "ref" in desc {
      let inner = follow(desc.inner, default: desc)
      if type(inner) == str {
        [#inner#sub[#desc.ref.join(" ")]]
      } else {
        desc.ref.join(" ")
      }
    } else if "op" in desc {
      desc.args.map(d => [(#show-desc(d))]).join([ #desc.op ])
    }
  }

  let at-field(page, field) = {
    docs.at(page, default: forms.at(page, default: none)).at(field)
  }

  // Normalizes a value, and captures `docs` and `forms`.
  // Returns a (value: decimal | string, unit: unit | none, desc: content)
  let normalize(v) = {
    let is-ref(ref) = { type(ref) == array and ref.len() == 2 }
    let is-raw(raw) = { type(raw) == str }
    let is-op(op) = { has-keys(op, ("op", "val-fn", "unit-fn", "args")) }

    if is-ref(v) {
      let page = v.at(0)
      let field = v.at(1)
      let v = at-field(page, field)
      let normalized = normalize(v)
      normalized.desc = (ref: (page, field), inner: normalized.desc)

      return normalized
    }

    if is-raw(v) {
      return new-value(parse: v)
    }

    if is-op(v) {
      let args = v.args.map(normalize)
      let value = (v.val-fn)(..args.map(arg => arg.value))
      let unit = (v.unit-fn)(..args.map(arg => arg.unit))
      let desc = (op: v.op, args: args.map(arg => arg.desc))

      return new-value(value: value, unit: unit, desc: desc)
    }

    if type(v) == dictionary {
      return new-value(..v)
    }

    panic("failed to normalize", v)
  }

  let impl-to-tables(dict) = {
    let tables = (:)
    for (name, page) in dict.pairs() {
      let rows = ()
      for (field, value) in page.pairs() {
        let value = normalize(value)
        let unit = show-unit(value.unit)
        let desc = show-desc(value.desc)
        rows.push((field, [ #value.value #unit ], desc))
      }

      [
        == #name
        #table(
          columns: 3,
          align: (left, right, left),
          [Field], [Value], [Description],
          ..rows.flatten()
        )
      ]
    }
  }

  (docs: impl-to-tables(docs), forms: impl-to-tables(forms))
}
