var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __decorateClass = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc(target, key) : target;
  for (var i4 = decorators.length - 1, decorator; i4 >= 0; i4--)
    if (decorator = decorators[i4])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result)
    __defProp(target, key, result);
  return result;
};

// ../node_modules/@lit/reactive-element/css-tag.js
var t = globalThis;
var e = t.ShadowRoot && (void 0 === t.ShadyCSS || t.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype;
var s = Symbol();
var o = /* @__PURE__ */ new WeakMap();
var n = class {
  constructor(t3, e5, o5) {
    if (this._$cssResult$ = true, o5 !== s)
      throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
    this.cssText = t3, this.t = e5;
  }
  get styleSheet() {
    let t3 = this.o;
    const s4 = this.t;
    if (e && void 0 === t3) {
      const e5 = void 0 !== s4 && 1 === s4.length;
      e5 && (t3 = o.get(s4)), void 0 === t3 && ((this.o = t3 = new CSSStyleSheet()).replaceSync(this.cssText), e5 && o.set(s4, t3));
    }
    return t3;
  }
  toString() {
    return this.cssText;
  }
};
var r = (t3) => new n("string" == typeof t3 ? t3 : t3 + "", void 0, s);
var i = (t3, ...e5) => {
  const o5 = 1 === t3.length ? t3[0] : e5.reduce((e6, s4, o6) => e6 + ((t4) => {
    if (true === t4._$cssResult$)
      return t4.cssText;
    if ("number" == typeof t4)
      return t4;
    throw Error("Value passed to 'css' function must be a 'css' function result: " + t4 + ". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.");
  })(s4) + t3[o6 + 1], t3[0]);
  return new n(o5, t3, s);
};
var S = (s4, o5) => {
  if (e)
    s4.adoptedStyleSheets = o5.map((t3) => t3 instanceof CSSStyleSheet ? t3 : t3.styleSheet);
  else
    for (const e5 of o5) {
      const o6 = document.createElement("style"), n5 = t.litNonce;
      void 0 !== n5 && o6.setAttribute("nonce", n5), o6.textContent = e5.cssText, s4.appendChild(o6);
    }
};
var c = e ? (t3) => t3 : (t3) => t3 instanceof CSSStyleSheet ? ((t4) => {
  let e5 = "";
  for (const s4 of t4.cssRules)
    e5 += s4.cssText;
  return r(e5);
})(t3) : t3;

// ../node_modules/@lit/reactive-element/reactive-element.js
var { is: i2, defineProperty: e2, getOwnPropertyDescriptor: r2, getOwnPropertyNames: h, getOwnPropertySymbols: o2, getPrototypeOf: n2 } = Object;
var a = globalThis;
var c2 = a.trustedTypes;
var l = c2 ? c2.emptyScript : "";
var p = a.reactiveElementPolyfillSupport;
var d = (t3, s4) => t3;
var u = { toAttribute(t3, s4) {
  switch (s4) {
    case Boolean:
      t3 = t3 ? l : null;
      break;
    case Object:
    case Array:
      t3 = null == t3 ? t3 : JSON.stringify(t3);
  }
  return t3;
}, fromAttribute(t3, s4) {
  let i4 = t3;
  switch (s4) {
    case Boolean:
      i4 = null !== t3;
      break;
    case Number:
      i4 = null === t3 ? null : Number(t3);
      break;
    case Object:
    case Array:
      try {
        i4 = JSON.parse(t3);
      } catch (t4) {
        i4 = null;
      }
  }
  return i4;
} };
var f = (t3, s4) => !i2(t3, s4);
var y = { attribute: true, type: String, converter: u, reflect: false, hasChanged: f };
Symbol.metadata ??= Symbol("metadata"), a.litPropertyMetadata ??= /* @__PURE__ */ new WeakMap();
var b = class extends HTMLElement {
  static addInitializer(t3) {
    this._$Ei(), (this.l ??= []).push(t3);
  }
  static get observedAttributes() {
    return this.finalize(), this._$Eh && [...this._$Eh.keys()];
  }
  static createProperty(t3, s4 = y) {
    if (s4.state && (s4.attribute = false), this._$Ei(), this.elementProperties.set(t3, s4), !s4.noAccessor) {
      const i4 = Symbol(), r6 = this.getPropertyDescriptor(t3, i4, s4);
      void 0 !== r6 && e2(this.prototype, t3, r6);
    }
  }
  static getPropertyDescriptor(t3, s4, i4) {
    const { get: e5, set: h3 } = r2(this.prototype, t3) ?? { get() {
      return this[s4];
    }, set(t4) {
      this[s4] = t4;
    } };
    return { get() {
      return e5?.call(this);
    }, set(s5) {
      const r6 = e5?.call(this);
      h3.call(this, s5), this.requestUpdate(t3, r6, i4);
    }, configurable: true, enumerable: true };
  }
  static getPropertyOptions(t3) {
    return this.elementProperties.get(t3) ?? y;
  }
  static _$Ei() {
    if (this.hasOwnProperty(d("elementProperties")))
      return;
    const t3 = n2(this);
    t3.finalize(), void 0 !== t3.l && (this.l = [...t3.l]), this.elementProperties = new Map(t3.elementProperties);
  }
  static finalize() {
    if (this.hasOwnProperty(d("finalized")))
      return;
    if (this.finalized = true, this._$Ei(), this.hasOwnProperty(d("properties"))) {
      const t4 = this.properties, s4 = [...h(t4), ...o2(t4)];
      for (const i4 of s4)
        this.createProperty(i4, t4[i4]);
    }
    const t3 = this[Symbol.metadata];
    if (null !== t3) {
      const s4 = litPropertyMetadata.get(t3);
      if (void 0 !== s4)
        for (const [t4, i4] of s4)
          this.elementProperties.set(t4, i4);
    }
    this._$Eh = /* @__PURE__ */ new Map();
    for (const [t4, s4] of this.elementProperties) {
      const i4 = this._$Eu(t4, s4);
      void 0 !== i4 && this._$Eh.set(i4, t4);
    }
    this.elementStyles = this.finalizeStyles(this.styles);
  }
  static finalizeStyles(s4) {
    const i4 = [];
    if (Array.isArray(s4)) {
      const e5 = new Set(s4.flat(1 / 0).reverse());
      for (const s5 of e5)
        i4.unshift(c(s5));
    } else
      void 0 !== s4 && i4.push(c(s4));
    return i4;
  }
  static _$Eu(t3, s4) {
    const i4 = s4.attribute;
    return false === i4 ? void 0 : "string" == typeof i4 ? i4 : "string" == typeof t3 ? t3.toLowerCase() : void 0;
  }
  constructor() {
    super(), this._$Ep = void 0, this.isUpdatePending = false, this.hasUpdated = false, this._$Em = null, this._$Ev();
  }
  _$Ev() {
    this._$ES = new Promise((t3) => this.enableUpdating = t3), this._$AL = /* @__PURE__ */ new Map(), this._$E_(), this.requestUpdate(), this.constructor.l?.forEach((t3) => t3(this));
  }
  addController(t3) {
    (this._$EO ??= /* @__PURE__ */ new Set()).add(t3), void 0 !== this.renderRoot && this.isConnected && t3.hostConnected?.();
  }
  removeController(t3) {
    this._$EO?.delete(t3);
  }
  _$E_() {
    const t3 = /* @__PURE__ */ new Map(), s4 = this.constructor.elementProperties;
    for (const i4 of s4.keys())
      this.hasOwnProperty(i4) && (t3.set(i4, this[i4]), delete this[i4]);
    t3.size > 0 && (this._$Ep = t3);
  }
  createRenderRoot() {
    const t3 = this.shadowRoot ?? this.attachShadow(this.constructor.shadowRootOptions);
    return S(t3, this.constructor.elementStyles), t3;
  }
  connectedCallback() {
    this.renderRoot ??= this.createRenderRoot(), this.enableUpdating(true), this._$EO?.forEach((t3) => t3.hostConnected?.());
  }
  enableUpdating(t3) {
  }
  disconnectedCallback() {
    this._$EO?.forEach((t3) => t3.hostDisconnected?.());
  }
  attributeChangedCallback(t3, s4, i4) {
    this._$AK(t3, i4);
  }
  _$EC(t3, s4) {
    const i4 = this.constructor.elementProperties.get(t3), e5 = this.constructor._$Eu(t3, i4);
    if (void 0 !== e5 && true === i4.reflect) {
      const r6 = (void 0 !== i4.converter?.toAttribute ? i4.converter : u).toAttribute(s4, i4.type);
      this._$Em = t3, null == r6 ? this.removeAttribute(e5) : this.setAttribute(e5, r6), this._$Em = null;
    }
  }
  _$AK(t3, s4) {
    const i4 = this.constructor, e5 = i4._$Eh.get(t3);
    if (void 0 !== e5 && this._$Em !== e5) {
      const t4 = i4.getPropertyOptions(e5), r6 = "function" == typeof t4.converter ? { fromAttribute: t4.converter } : void 0 !== t4.converter?.fromAttribute ? t4.converter : u;
      this._$Em = e5, this[e5] = r6.fromAttribute(s4, t4.type), this._$Em = null;
    }
  }
  requestUpdate(t3, s4, i4) {
    if (void 0 !== t3) {
      if (i4 ??= this.constructor.getPropertyOptions(t3), !(i4.hasChanged ?? f)(this[t3], s4))
        return;
      this.P(t3, s4, i4);
    }
    false === this.isUpdatePending && (this._$ES = this._$ET());
  }
  P(t3, s4, i4) {
    this._$AL.has(t3) || this._$AL.set(t3, s4), true === i4.reflect && this._$Em !== t3 && (this._$Ej ??= /* @__PURE__ */ new Set()).add(t3);
  }
  async _$ET() {
    this.isUpdatePending = true;
    try {
      await this._$ES;
    } catch (t4) {
      Promise.reject(t4);
    }
    const t3 = this.scheduleUpdate();
    return null != t3 && await t3, !this.isUpdatePending;
  }
  scheduleUpdate() {
    return this.performUpdate();
  }
  performUpdate() {
    if (!this.isUpdatePending)
      return;
    if (!this.hasUpdated) {
      if (this.renderRoot ??= this.createRenderRoot(), this._$Ep) {
        for (const [t5, s5] of this._$Ep)
          this[t5] = s5;
        this._$Ep = void 0;
      }
      const t4 = this.constructor.elementProperties;
      if (t4.size > 0)
        for (const [s5, i4] of t4)
          true !== i4.wrapped || this._$AL.has(s5) || void 0 === this[s5] || this.P(s5, this[s5], i4);
    }
    let t3 = false;
    const s4 = this._$AL;
    try {
      t3 = this.shouldUpdate(s4), t3 ? (this.willUpdate(s4), this._$EO?.forEach((t4) => t4.hostUpdate?.()), this.update(s4)) : this._$EU();
    } catch (s5) {
      throw t3 = false, this._$EU(), s5;
    }
    t3 && this._$AE(s4);
  }
  willUpdate(t3) {
  }
  _$AE(t3) {
    this._$EO?.forEach((t4) => t4.hostUpdated?.()), this.hasUpdated || (this.hasUpdated = true, this.firstUpdated(t3)), this.updated(t3);
  }
  _$EU() {
    this._$AL = /* @__PURE__ */ new Map(), this.isUpdatePending = false;
  }
  get updateComplete() {
    return this.getUpdateComplete();
  }
  getUpdateComplete() {
    return this._$ES;
  }
  shouldUpdate(t3) {
    return true;
  }
  update(t3) {
    this._$Ej &&= this._$Ej.forEach((t4) => this._$EC(t4, this[t4])), this._$EU();
  }
  updated(t3) {
  }
  firstUpdated(t3) {
  }
};
b.elementStyles = [], b.shadowRootOptions = { mode: "open" }, b[d("elementProperties")] = /* @__PURE__ */ new Map(), b[d("finalized")] = /* @__PURE__ */ new Map(), p?.({ ReactiveElement: b }), (a.reactiveElementVersions ??= []).push("2.0.4");

// ../node_modules/lit-html/lit-html.js
var t2 = globalThis;
var i3 = t2.trustedTypes;
var s2 = i3 ? i3.createPolicy("lit-html", { createHTML: (t3) => t3 }) : void 0;
var e3 = "$lit$";
var h2 = `lit$${Math.random().toFixed(9).slice(2)}$`;
var o3 = "?" + h2;
var n3 = `<${o3}>`;
var r3 = document;
var l2 = () => r3.createComment("");
var c3 = (t3) => null === t3 || "object" != typeof t3 && "function" != typeof t3;
var a2 = Array.isArray;
var u2 = (t3) => a2(t3) || "function" == typeof t3?.[Symbol.iterator];
var d2 = "[ 	\n\f\r]";
var f2 = /<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g;
var v = /-->/g;
var _ = />/g;
var m = RegExp(`>|${d2}(?:([^\\s"'>=/]+)(${d2}*=${d2}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`, "g");
var p2 = /'/g;
var g = /"/g;
var $ = /^(?:script|style|textarea|title)$/i;
var y2 = (t3) => (i4, ...s4) => ({ _$litType$: t3, strings: i4, values: s4 });
var x = y2(1);
var b2 = y2(2);
var w = Symbol.for("lit-noChange");
var T = Symbol.for("lit-nothing");
var A = /* @__PURE__ */ new WeakMap();
var E = r3.createTreeWalker(r3, 129);
function C(t3, i4) {
  if (!Array.isArray(t3) || !t3.hasOwnProperty("raw"))
    throw Error("invalid template strings array");
  return void 0 !== s2 ? s2.createHTML(i4) : i4;
}
var P = (t3, i4) => {
  const s4 = t3.length - 1, o5 = [];
  let r6, l3 = 2 === i4 ? "<svg>" : "", c4 = f2;
  for (let i5 = 0; i5 < s4; i5++) {
    const s5 = t3[i5];
    let a3, u3, d3 = -1, y3 = 0;
    for (; y3 < s5.length && (c4.lastIndex = y3, u3 = c4.exec(s5), null !== u3); )
      y3 = c4.lastIndex, c4 === f2 ? "!--" === u3[1] ? c4 = v : void 0 !== u3[1] ? c4 = _ : void 0 !== u3[2] ? ($.test(u3[2]) && (r6 = RegExp("</" + u3[2], "g")), c4 = m) : void 0 !== u3[3] && (c4 = m) : c4 === m ? ">" === u3[0] ? (c4 = r6 ?? f2, d3 = -1) : void 0 === u3[1] ? d3 = -2 : (d3 = c4.lastIndex - u3[2].length, a3 = u3[1], c4 = void 0 === u3[3] ? m : '"' === u3[3] ? g : p2) : c4 === g || c4 === p2 ? c4 = m : c4 === v || c4 === _ ? c4 = f2 : (c4 = m, r6 = void 0);
    const x2 = c4 === m && t3[i5 + 1].startsWith("/>") ? " " : "";
    l3 += c4 === f2 ? s5 + n3 : d3 >= 0 ? (o5.push(a3), s5.slice(0, d3) + e3 + s5.slice(d3) + h2 + x2) : s5 + h2 + (-2 === d3 ? i5 : x2);
  }
  return [C(t3, l3 + (t3[s4] || "<?>") + (2 === i4 ? "</svg>" : "")), o5];
};
var V = class _V {
  constructor({ strings: t3, _$litType$: s4 }, n5) {
    let r6;
    this.parts = [];
    let c4 = 0, a3 = 0;
    const u3 = t3.length - 1, d3 = this.parts, [f3, v2] = P(t3, s4);
    if (this.el = _V.createElement(f3, n5), E.currentNode = this.el.content, 2 === s4) {
      const t4 = this.el.content.firstChild;
      t4.replaceWith(...t4.childNodes);
    }
    for (; null !== (r6 = E.nextNode()) && d3.length < u3; ) {
      if (1 === r6.nodeType) {
        if (r6.hasAttributes())
          for (const t4 of r6.getAttributeNames())
            if (t4.endsWith(e3)) {
              const i4 = v2[a3++], s5 = r6.getAttribute(t4).split(h2), e5 = /([.?@])?(.*)/.exec(i4);
              d3.push({ type: 1, index: c4, name: e5[2], strings: s5, ctor: "." === e5[1] ? k : "?" === e5[1] ? H : "@" === e5[1] ? I : R }), r6.removeAttribute(t4);
            } else
              t4.startsWith(h2) && (d3.push({ type: 6, index: c4 }), r6.removeAttribute(t4));
        if ($.test(r6.tagName)) {
          const t4 = r6.textContent.split(h2), s5 = t4.length - 1;
          if (s5 > 0) {
            r6.textContent = i3 ? i3.emptyScript : "";
            for (let i4 = 0; i4 < s5; i4++)
              r6.append(t4[i4], l2()), E.nextNode(), d3.push({ type: 2, index: ++c4 });
            r6.append(t4[s5], l2());
          }
        }
      } else if (8 === r6.nodeType)
        if (r6.data === o3)
          d3.push({ type: 2, index: c4 });
        else {
          let t4 = -1;
          for (; -1 !== (t4 = r6.data.indexOf(h2, t4 + 1)); )
            d3.push({ type: 7, index: c4 }), t4 += h2.length - 1;
        }
      c4++;
    }
  }
  static createElement(t3, i4) {
    const s4 = r3.createElement("template");
    return s4.innerHTML = t3, s4;
  }
};
function N(t3, i4, s4 = t3, e5) {
  if (i4 === w)
    return i4;
  let h3 = void 0 !== e5 ? s4._$Co?.[e5] : s4._$Cl;
  const o5 = c3(i4) ? void 0 : i4._$litDirective$;
  return h3?.constructor !== o5 && (h3?._$AO?.(false), void 0 === o5 ? h3 = void 0 : (h3 = new o5(t3), h3._$AT(t3, s4, e5)), void 0 !== e5 ? (s4._$Co ??= [])[e5] = h3 : s4._$Cl = h3), void 0 !== h3 && (i4 = N(t3, h3._$AS(t3, i4.values), h3, e5)), i4;
}
var S2 = class {
  constructor(t3, i4) {
    this._$AV = [], this._$AN = void 0, this._$AD = t3, this._$AM = i4;
  }
  get parentNode() {
    return this._$AM.parentNode;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  u(t3) {
    const { el: { content: i4 }, parts: s4 } = this._$AD, e5 = (t3?.creationScope ?? r3).importNode(i4, true);
    E.currentNode = e5;
    let h3 = E.nextNode(), o5 = 0, n5 = 0, l3 = s4[0];
    for (; void 0 !== l3; ) {
      if (o5 === l3.index) {
        let i5;
        2 === l3.type ? i5 = new M(h3, h3.nextSibling, this, t3) : 1 === l3.type ? i5 = new l3.ctor(h3, l3.name, l3.strings, this, t3) : 6 === l3.type && (i5 = new L(h3, this, t3)), this._$AV.push(i5), l3 = s4[++n5];
      }
      o5 !== l3?.index && (h3 = E.nextNode(), o5++);
    }
    return E.currentNode = r3, e5;
  }
  p(t3) {
    let i4 = 0;
    for (const s4 of this._$AV)
      void 0 !== s4 && (void 0 !== s4.strings ? (s4._$AI(t3, s4, i4), i4 += s4.strings.length - 2) : s4._$AI(t3[i4])), i4++;
  }
};
var M = class _M {
  get _$AU() {
    return this._$AM?._$AU ?? this._$Cv;
  }
  constructor(t3, i4, s4, e5) {
    this.type = 2, this._$AH = T, this._$AN = void 0, this._$AA = t3, this._$AB = i4, this._$AM = s4, this.options = e5, this._$Cv = e5?.isConnected ?? true;
  }
  get parentNode() {
    let t3 = this._$AA.parentNode;
    const i4 = this._$AM;
    return void 0 !== i4 && 11 === t3?.nodeType && (t3 = i4.parentNode), t3;
  }
  get startNode() {
    return this._$AA;
  }
  get endNode() {
    return this._$AB;
  }
  _$AI(t3, i4 = this) {
    t3 = N(this, t3, i4), c3(t3) ? t3 === T || null == t3 || "" === t3 ? (this._$AH !== T && this._$AR(), this._$AH = T) : t3 !== this._$AH && t3 !== w && this._(t3) : void 0 !== t3._$litType$ ? this.$(t3) : void 0 !== t3.nodeType ? this.T(t3) : u2(t3) ? this.k(t3) : this._(t3);
  }
  S(t3) {
    return this._$AA.parentNode.insertBefore(t3, this._$AB);
  }
  T(t3) {
    this._$AH !== t3 && (this._$AR(), this._$AH = this.S(t3));
  }
  _(t3) {
    this._$AH !== T && c3(this._$AH) ? this._$AA.nextSibling.data = t3 : this.T(r3.createTextNode(t3)), this._$AH = t3;
  }
  $(t3) {
    const { values: i4, _$litType$: s4 } = t3, e5 = "number" == typeof s4 ? this._$AC(t3) : (void 0 === s4.el && (s4.el = V.createElement(C(s4.h, s4.h[0]), this.options)), s4);
    if (this._$AH?._$AD === e5)
      this._$AH.p(i4);
    else {
      const t4 = new S2(e5, this), s5 = t4.u(this.options);
      t4.p(i4), this.T(s5), this._$AH = t4;
    }
  }
  _$AC(t3) {
    let i4 = A.get(t3.strings);
    return void 0 === i4 && A.set(t3.strings, i4 = new V(t3)), i4;
  }
  k(t3) {
    a2(this._$AH) || (this._$AH = [], this._$AR());
    const i4 = this._$AH;
    let s4, e5 = 0;
    for (const h3 of t3)
      e5 === i4.length ? i4.push(s4 = new _M(this.S(l2()), this.S(l2()), this, this.options)) : s4 = i4[e5], s4._$AI(h3), e5++;
    e5 < i4.length && (this._$AR(s4 && s4._$AB.nextSibling, e5), i4.length = e5);
  }
  _$AR(t3 = this._$AA.nextSibling, i4) {
    for (this._$AP?.(false, true, i4); t3 && t3 !== this._$AB; ) {
      const i5 = t3.nextSibling;
      t3.remove(), t3 = i5;
    }
  }
  setConnected(t3) {
    void 0 === this._$AM && (this._$Cv = t3, this._$AP?.(t3));
  }
};
var R = class {
  get tagName() {
    return this.element.tagName;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  constructor(t3, i4, s4, e5, h3) {
    this.type = 1, this._$AH = T, this._$AN = void 0, this.element = t3, this.name = i4, this._$AM = e5, this.options = h3, s4.length > 2 || "" !== s4[0] || "" !== s4[1] ? (this._$AH = Array(s4.length - 1).fill(new String()), this.strings = s4) : this._$AH = T;
  }
  _$AI(t3, i4 = this, s4, e5) {
    const h3 = this.strings;
    let o5 = false;
    if (void 0 === h3)
      t3 = N(this, t3, i4, 0), o5 = !c3(t3) || t3 !== this._$AH && t3 !== w, o5 && (this._$AH = t3);
    else {
      const e6 = t3;
      let n5, r6;
      for (t3 = h3[0], n5 = 0; n5 < h3.length - 1; n5++)
        r6 = N(this, e6[s4 + n5], i4, n5), r6 === w && (r6 = this._$AH[n5]), o5 ||= !c3(r6) || r6 !== this._$AH[n5], r6 === T ? t3 = T : t3 !== T && (t3 += (r6 ?? "") + h3[n5 + 1]), this._$AH[n5] = r6;
    }
    o5 && !e5 && this.j(t3);
  }
  j(t3) {
    t3 === T ? this.element.removeAttribute(this.name) : this.element.setAttribute(this.name, t3 ?? "");
  }
};
var k = class extends R {
  constructor() {
    super(...arguments), this.type = 3;
  }
  j(t3) {
    this.element[this.name] = t3 === T ? void 0 : t3;
  }
};
var H = class extends R {
  constructor() {
    super(...arguments), this.type = 4;
  }
  j(t3) {
    this.element.toggleAttribute(this.name, !!t3 && t3 !== T);
  }
};
var I = class extends R {
  constructor(t3, i4, s4, e5, h3) {
    super(t3, i4, s4, e5, h3), this.type = 5;
  }
  _$AI(t3, i4 = this) {
    if ((t3 = N(this, t3, i4, 0) ?? T) === w)
      return;
    const s4 = this._$AH, e5 = t3 === T && s4 !== T || t3.capture !== s4.capture || t3.once !== s4.once || t3.passive !== s4.passive, h3 = t3 !== T && (s4 === T || e5);
    e5 && this.element.removeEventListener(this.name, this, s4), h3 && this.element.addEventListener(this.name, this, t3), this._$AH = t3;
  }
  handleEvent(t3) {
    "function" == typeof this._$AH ? this._$AH.call(this.options?.host ?? this.element, t3) : this._$AH.handleEvent(t3);
  }
};
var L = class {
  constructor(t3, i4, s4) {
    this.element = t3, this.type = 6, this._$AN = void 0, this._$AM = i4, this.options = s4;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AI(t3) {
    N(this, t3);
  }
};
var Z = t2.litHtmlPolyfillSupport;
Z?.(V, M), (t2.litHtmlVersions ??= []).push("3.1.3");
var j = (t3, i4, s4) => {
  const e5 = s4?.renderBefore ?? i4;
  let h3 = e5._$litPart$;
  if (void 0 === h3) {
    const t4 = s4?.renderBefore ?? null;
    e5._$litPart$ = h3 = new M(i4.insertBefore(l2(), t4), t4, void 0, s4 ?? {});
  }
  return h3._$AI(t3), h3;
};

// ../node_modules/lit-element/lit-element.js
var s3 = class extends b {
  constructor() {
    super(...arguments), this.renderOptions = { host: this }, this._$Do = void 0;
  }
  createRenderRoot() {
    const t3 = super.createRenderRoot();
    return this.renderOptions.renderBefore ??= t3.firstChild, t3;
  }
  update(t3) {
    const i4 = this.render();
    this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(t3), this._$Do = j(i4, this.renderRoot, this.renderOptions);
  }
  connectedCallback() {
    super.connectedCallback(), this._$Do?.setConnected(true);
  }
  disconnectedCallback() {
    super.disconnectedCallback(), this._$Do?.setConnected(false);
  }
  render() {
    return w;
  }
};
s3._$litElement$ = true, s3["finalized", "finalized"] = true, globalThis.litElementHydrateSupport?.({ LitElement: s3 });
var r4 = globalThis.litElementPolyfillSupport;
r4?.({ LitElement: s3 });
(globalThis.litElementVersions ??= []).push("4.0.5");

// ../node_modules/@lit/reactive-element/decorators/property.js
var o4 = { attribute: true, type: String, converter: u, reflect: false, hasChanged: f };
var r5 = (t3 = o4, e5, r6) => {
  const { kind: n5, metadata: i4 } = r6;
  let s4 = globalThis.litPropertyMetadata.get(i4);
  if (void 0 === s4 && globalThis.litPropertyMetadata.set(i4, s4 = /* @__PURE__ */ new Map()), s4.set(r6.name, t3), "accessor" === n5) {
    const { name: o5 } = r6;
    return { set(r7) {
      const n6 = e5.get.call(this);
      e5.set.call(this, r7), this.requestUpdate(o5, n6, t3);
    }, init(e6) {
      return void 0 !== e6 && this.P(o5, void 0, t3), e6;
    } };
  }
  if ("setter" === n5) {
    const { name: o5 } = r6;
    return function(r7) {
      const n6 = this[o5];
      e5.call(this, r7), this.requestUpdate(o5, n6, t3);
    };
  }
  throw Error("Unsupported decorator location: " + n5);
};
function n4(t3) {
  return (e5, o5) => "object" == typeof o5 ? r5(t3, e5, o5) : ((t4, e6, o6) => {
    const r6 = e6.hasOwnProperty(o6);
    return e6.constructor.createProperty(o6, r6 ? { ...t4, wrapped: true } : t4), r6 ? Object.getOwnPropertyDescriptor(e6, o6) : void 0;
  })(t3, e5, o5);
}

// src/components/bs-debug.ts
var BsDebug = class extends s3 {
  constructor() {
    super(...arguments);
    this.servers = { servers: [] };
    this.me = { routes: [], id: "" };
  }
  get otherServers() {
    return this.servers.servers.filter((server) => server.id !== this.me.id);
  }
  render() {
    return x`
        <bs-header></bs-header>
        <bs-server-detail .server=${this.me}></bs-server-detail>
        ${this.otherServers.length > 0 ? x`
            <bs-server-list .servers=${this.otherServers}></bs-server-list>` : null}
    `;
  }
};
__decorateClass([
  n4({ type: Object })
], BsDebug.prototype, "servers", 2);
__decorateClass([
  n4({ type: Object })
], BsDebug.prototype, "me", 2);
customElements.define("bs-debug", BsDebug);

// styles/base.css.ts
var base = i`
  pre {
      margin: 0
  }
  a {
    color: var(--theme-txt-color);
    &:hover {
        text-decoration: none;
    }
  }
  p {
    margin: 0;
    padding: 0;
  }
`;

// src/components/bs-server-list.ts
var BsServerList = class extends s3 {
  constructor() {
    super(...arguments);
    this.servers = [];
  }
  static {
    this.styles = [
      base,
      i`
    
    `
    ];
  }
  render() {
    return x`
        ${this.servers.map((server) => {
      const display_addr = "http://" + server.socket_addr;
      let url = new URL(display_addr);
      let bs_url = new URL("./__bslive", display_addr);
      return x`
                        <div>
                            <bs-server-identity .identity=${server.identity}></bs-server-identity>
                            <p><a href=${url} target="_blank"><code>${url}</code></a></p>
                            <p>
                                <bs-icon icon-name="logo"></bs-icon>
                                <small><a href=${bs_url} target="_blank"><code>${bs_url}</code></a></small>
                            </p>
                        </div>
                    `;
    })}
    `;
  }
};
__decorateClass([
  n4({ type: Object })
], BsServerList.prototype, "servers", 2);
customElements.define("bs-server-list", BsServerList);

// src/components/bs-server-detail.ts
var BsServerDetail = class extends s3 {
  constructor() {
    super(...arguments);
    this.server = { routes: [], id: "" };
  }
  static {
    this.styles = [
      base
    ];
  }
  render() {
    return x`
       <pre><code>${JSON.stringify(this.server, null, 2)}</code></pre>
    `;
  }
};
__decorateClass([
  n4({ type: Object })
], BsServerDetail.prototype, "server", 2);
customElements.define("bs-server-detail", BsServerDetail);

// src/components/bs-server-identity.ts
var BsServerIdentity = class extends s3 {
  static {
    this.styles = [base];
  }
  render() {
    switch (this.identity.kind) {
      case "Named":
      case "Both": {
        return x`<p><strong>[named] ${this.identity.payload.name}</strong></p>`;
      }
      default:
        return x`<p><strong>[unnamed]</strong></p>`;
    }
  }
};
__decorateClass([
  n4({ type: Object })
], BsServerIdentity.prototype, "identity", 2);
customElements.define("bs-server-identity", BsServerIdentity);

// src/components/bs-header.ts
var BsHeader = class extends s3 {
  constructor() {
    super(...arguments);
    this.servers = [];
  }
  static {
    this.styles = [
      base,
      i`
    .logo {
        position: relative;
        color: var(--theme-txt-color);  
    }   
    .logo bs-icon::part(svg) {
        height: 30px;
        width: 140px;
    }
    `
    ];
  }
  render() {
    return x`
        <div class="logo">
            <bs-icon icon-name="wordmark"></bs-icon>
        </div>
    `;
  }
};
__decorateClass([
  n4({ type: Object })
], BsHeader.prototype, "servers", 2);
customElements.define("bs-header", BsHeader);

// src/components/bs-icon.ts
var BsIcon = class extends s3 {
  static {
    this.styles = [
      base,
      i`
    .svg-icon {
        display: inline-block;
        fill: var(--bs-icon-color, currentColor);
        height: var(--bs-icon-height, 1em);
        width: var(--bs-icon-width, 1em);
        vertical-align: middle;
    }
    `
    ];
  }
  get icon() {
    switch (this.iconName) {
      case "logo":
        return x`<svg class="svg-icon" part="svg"><use xlink:href="#svg-logo"></use></svg>`;
      case "wordmark":
        return x`<svg class="svg-icon" part="svg"><use xlink:href="#svg-wordmark"></use></svg>`;
      default:
        return `unknown`;
    }
  }
  render() {
    return x`
        <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" style="display:none">
            <symbol id="svg-check" viewBox="0 0 20 20">
                <path d="M8.294 16.998c-.435 0-.847-.203-1.11-.553l-3.574-4.72c-.465-.614-.344-1.487.27-1.952.615-.467 1.488-.344 1.953.27l2.35 3.104 5.912-9.492c.407-.652 1.267-.852 1.92-.445.654.406.855 1.266.447 1.92L9.478 16.34c-.242.39-.66.635-1.12.656-.022.002-.042.002-.064.002z"/>
            </symbol>
            <symbol id="svg-creative-commons-noncommercial-us" viewBox="0 0 20 20">
                <path d="M9.988.4c2.69 0 4.966.928 6.825 2.784C18.67 5.04 19.6 7.312 19.6 10s-.913 4.936-2.74 6.744C14.923 18.648 12.63 19.6 9.99 19.6c-2.61 0-4.862-.944-6.753-2.832C1.345 14.88.4 12.624.4 10s.945-4.896 2.835-6.816C5.078 1.328 7.33.4 9.988.4zM2.56 7.42c-.287.81-.43 1.67-.43 2.58 0 2.128.777 3.968 2.33 5.52 1.555 1.552 3.405 2.328 5.552 2.328s4.013-.784 5.6-2.352c.53-.513.967-1.073 1.31-1.68l-3.618-1.61c-.246 1.216-1.33 2.04-2.643 2.136v1.48h-1.1v-1.48c-1.078-.013-2.12-.453-2.915-1.15l1.322-1.333c.637.598 1.274.868 2.143.868.563 0 1.188-.22 1.188-.955 0-.26-.1-.44-.26-.577l-.915-.407-1.14-.508c-.563-.252-1.04-.464-1.52-.677L2.56 7.42zm7.452-5.292c-2.18 0-4.02.768-5.527 2.304-.41.414-.766.846-1.07 1.297l3.67 1.632c.332-1.017 1.3-1.635 2.474-1.704v-1.48h1.1v1.48c.76.037 1.593.245 2.413.88l-1.26 1.297c-.466-.33-1.054-.563-1.642-.563-.476 0-1.15.148-1.15.747 0 .09.03.17.086.242l1.228.547.83.37c.532.236 1.04.46 1.542.685l4.92 2.19c.162-.644.244-1.33.244-2.055 0-2.192-.77-4.048-2.307-5.568-1.522-1.536-3.372-2.304-5.55-2.304z"/>
            </symbol>
            <symbol id="svg-back-in-time" viewBox="0 0 20 20">
                <path d="M11 1.8c-4.445 0-8.06 3.56-8.17 7.995V10H.46l3.593 3.894L7.547 10H4.875v-.205C4.982 6.492 7.683 3.85 11 3.85c3.386 0 6.13 2.754 6.13 6.15 0 3.396-2.744 6.15-6.13 6.15-1.357 0-2.61-.445-3.627-1.193L5.967 16.46C7.355 17.55 9.102 18.2 11 18.2c4.515 0 8.174-3.67 8.174-8.2S15.514 1.8 11 1.8zM10 5v5c0 .13.027.26.077.382s.124.233.216.325l3.2 3.2c.283-.183.55-.39.787-.628L12 11V5h-2z"/>
            </symbol>
            <symbol id="svg-time-slot" viewBox="0 0 20 20">
                <path d="M10 .4C4.698.4.4 4.698.4 10s4.298 9.6 9.6 9.6c5.3 0 9.6-4.298 9.6-9.6S15.3.4 10 .4zm0 17.2c-4.197 0-7.6-3.403-7.6-7.6C2.4 5.8 5.802 2.4 10 2.4V10l6.792-3.396c.513 1.023.808 2.173.808 3.396 0 4.197-3.403 7.6-7.6 7.6z"/>
            </symbol>
            <symbol id="svg-merge" viewBox="0 0 20 20">
                <path d="M17.89 17.707L16.892 20c-3.137-1.366-5.496-3.152-6.892-5.275-1.396 2.123-3.755 3.91-6.892 5.275l-.998-2.293C5.14 16.39 8.55 14.102 8.55 10V7H5.5L10 0l4.5 7h-3.05v3c0 4.102 3.41 6.39 6.44 7.707z"/>
            </symbol>
            <symbol id="svg-text" viewBox="0 0 20 20">
                <path fill-rule="evenodd" clip-rule="evenodd" d="M15.5 11h-11c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h11c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5zm0-4h-11c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h11c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5zm-5 8h-6c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h6c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5zm5-12h-11c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h11c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5z"/>
            </symbol>
            <symbol id="svg-tv" viewBox="0 0 20 20">
                <path d="M18 1H2C.9 1 0 1.9 0 3v11c0 1.1.882 2.178 1.96 2.393l4.373.875S2.57 19 5 19h10c2.43 0-1.334-1.732-1.334-1.732l4.373-.875C19.116 16.178 20 15.1 20 14V3c0-1.1-.9-2-2-2zm0 13H2V3h16v11z"/>
            </symbol>
            <symbol id="svg-block" viewBox="0 0 20 20">
                <path d="M10 .4C4.697.4.4 4.698.4 10c0 5.303 4.297 9.6 9.6 9.6 5.3 0 9.6-4.297 9.6-9.6 0-5.302-4.3-9.6-9.6-9.6zM2.4 10c0-4.197 3.4-7.6 7.6-7.6 1.828 0 3.505.647 4.816 1.723L4.122 14.817C3.046 13.505 2.4 11.83 2.4 10zm7.6 7.6c-1.83 0-3.506-.647-4.816-1.723L15.878 5.184C16.953 6.496 17.6 8.17 17.6 10c0 4.197-3.404 7.6-7.6 7.6z"/>
            </symbol>
            <symbol id="svg-list" viewBox="0 0 20 20">
                <path d="M14.4 9H8.6c-.552 0-.6.447-.6 1s.048 1 .6 1h5.8c.552 0 .6-.447.6-1s-.048-1-.6-1zm2 5H8.6c-.552 0-.6.447-.6 1s.048 1 .6 1h7.8c.552 0 .6-.447.6-1s-.048-1-.6-1zM8.6 6h7.8c.552 0 .6-.447.6-1s-.048-1-.6-1H8.6c-.552 0-.6.447-.6 1s.048 1 .6 1zM5.4 9H3.6c-.552 0-.6.447-.6 1s.048 1 .6 1h1.8c.552 0 .6-.447.6-1s-.048-1-.6-1zm0 5H3.6c-.552 0-.6.447-.6 1s.048 1 .6 1h1.8c.552 0 .6-.447.6-1s-.048-1-.6-1zm0-10H3.6c-.552 0-.6.447-.6 1s.048 1 .6 1h1.8c.552 0 .6-.447.6-1s-.048-1-.6-1z"/>
            </symbol>
            <symbol id="svg-logo" viewBox="0 0 140 204.1">
                <path d="M63.5.3L1.7 31.2c-1 .5-1.7 1.5-1.7 2.7v136.3c0 1.1.6 2.2 1.7 2.7l61.8 30.9c2 1 4.3-.5 4.3-2.7V3c0-2.2-2.3-3.7-4.3-2.7zM76.5 203.8l61.8-30.9c1-.5 1.7-1.5 1.7-2.7v-66.3c0-1.1-.6-2.2-1.7-2.7L76.5 70.3c-2-1-4.3.5-4.3 2.7v128.1c0 2.2 2.3 3.7 4.3 2.7z"/>
            </symbol>
            <symbol id="svg-wordmark" viewBox="0 0 536.3 106.8">
                <path d="M33 .2L.9 16.2c-.6.3-.9.8-.9 1.4v70.8c0 .6.3 1.1.9 1.4l32.1 16c1 .5 2.3-.2 2.3-1.4V1.6C35.2.4 34-.4 33 .2zM39.7 105.8l32.1-16c.5-.3.9-.8.9-1.4V54c0-.6-.3-1.1-.9-1.4l-32.1-16c-1-.5-2.3.2-2.3 1.4v66.5c.1 1.1 1.3 1.8 2.3 1.3zM129.7 34.8c10.8 0 16.6 4 16.6 14.1 0 6.6-2.1 9.8-6.4 12.2 4.7 1.8 7.8 5.2 7.8 12.6 0 11.1-6.7 15.4-17.3 15.4H109V34.8h20.7zm-11.8 7.6V58h11.7c5.4 0 7.8-2.7 7.8-8 0-5.2-2.7-7.5-8.1-7.5h-11.4zm0 23v16.1h12c5.5 0 8.7-1.7 8.7-8.3 0-6.2-4.6-7.9-8.9-7.9h-11.8zM156.6 49.5h8.6v4.8s6.7-4.4 13.5-5.6v8.6c-7.2 1.4-13.4 6.3-13.4 6.3v25.6h-8.6V49.5zM365.4 49.5h8.6v4.8s6.7-4.4 13.5-5.6v8.6c-7.2 1.4-13.4 6.3-13.4 6.3v25.6h-8.6V49.5zM218.4 69.1c0 13.2-4 20.9-17.7 20.9-13.6 0-17.7-7.8-17.7-20.9 0-12.9 4.4-20.5 17.7-20.5s17.7 7.6 17.7 20.5zm-8.7 0c0-9.2-2-13.2-9-13.2s-9 4-9 13.2 1.6 13.6 9 13.6 9-4.4 9-13.6zM232.3 49.5l6.3 32.3h1.6l7.5-31.5h8.9l7.5 31.5h1.6l6.2-32.3h8.6l-8.4 39.7h-13.7L252.2 62 246 89.2h-13.7l-8.4-39.7h8.4zM315.4 57.7s-9.4-1.3-14.1-1.3c-4.8 0-6.9 1.1-6.9 4.4 0 2.6 1.7 3.3 9.4 4.7 9.5 1.7 12.9 4 12.9 12 0 9.3-5.9 12.6-15.7 12.6-5.5 0-14.7-1.7-14.7-1.7l.3-7.2s9.5 1.3 13.6 1.3c5.7 0 7.9-1.2 7.9-4.7 0-2.8-1.3-3.6-9.2-4.9-8.7-1.4-13.2-3.3-13.2-11.7 0-9 7-12.3 14.8-12.3 5.8 0 14.9 1.7 14.9 1.7v7.1zM355.6 81.8l.2 6.4s-9 1.8-16 1.8c-11.9 0-16.5-6.3-16.5-20.3 0-14.5 6.3-21.1 17.2-21.1 11.1 0 16.7 5.8 16.7 18.2l-.6 6.2H332c.1 6.3 2.5 9.5 9 9.5 6.2 0 14.6-.7 14.6-.7zm-7-15.5c0-7.9-2.4-10.6-8.2-10.6-5.9 0-8.5 2.9-8.6 10.6h16.8zM420.5 54.3S412 53 406.8 53c-4.9 0-9.4 1.3-9.4 6.7 0 4.1 2 5.3 10.6 6.7 10.2 1.7 14 3.5 14 11.1 0 9.3-5.8 12.3-15.3 12.3-4.8 0-13.6-1.4-13.6-1.4l.3-4.2s8.9 1.3 12.9 1.3c6.8 0 10.8-1.6 10.8-7.8 0-4.8-2.4-5.8-11.3-7.1-9.1-1.4-13.3-3.1-13.3-10.8 0-8.6 7.1-11.2 14-11.2 6 0 14 1.3 14 1.3v4.4zM432 49.5L442.5 85h2.9L456 49.5h4.8l-16.9 57.3h-4.8l5.1-17.7h-5.5L427 49.4h5zM468.9 89.2V49.5h4.7v2.9s6.7-3.7 12.9-3.7c10.9 0 13.3 5.1 13.3 19.6v20.9H495V68.5c0-11.7-1.3-15.6-9.2-15.6-6.2 0-12.2 3.3-12.2 3.3V89h-4.7zM536.2 49.7l-.2 4s-6.3-.8-9.3-.8c-9.5 0-12.3 4.2-12.3 15.6 0 12.5 1.9 17.1 12.3 17.1 3 0 9.4-.7 9.4-.7l.2 4s-7.1 1-10.5 1c-12.9 0-16.3-5.7-16.3-21.3 0-14.5 4.6-19.9 16.4-19.9 3.4 0 10.3 1 10.3 1z"/>
            </symbol>
            <symbol id="svg-github" viewBox="0 0 32 32">
                <path clip-rule="evenodd" d="M16.003 0C7.17 0 .008 7.162.008 15.997c0 7.067 4.582 13.063 10.94 15.18.8.145 1.052-.33 1.052-.753 0-.38.008-1.442 0-2.777-4.45.967-5.37-2.107-5.37-2.107-.728-1.848-1.776-2.34-1.776-2.34-1.452-.992.11-.973.11-.973 1.604.113 2.45 1.65 2.45 1.65 1.427 2.442 3.743 1.736 4.654 1.328.146-1.034.56-1.74 1.017-2.14C9.533 22.663 5.8 21.29 5.8 15.16c0-1.747.622-3.174 1.645-4.292-.165-.404-.715-2.03.157-4.234 0 0 1.343-.43 4.398 1.64 1.276-.354 2.645-.53 4.005-.537 1.36.006 2.727.183 4.005.538 3.055-2.07 4.396-1.64 4.396-1.64.872 2.202.323 3.83.16 4.233 1.022 1.118 1.643 2.545 1.643 4.292 0 6.146-3.74 7.498-7.305 7.893C19.48 23.548 20 24.508 20 26v4.428c0 .428.258.9 1.07.746C27.422 29.054 32 23.062 32 15.997 32 7.162 24.838 0 16.003 0z" fill-rule="evenodd"/>
            </symbol>
            <symbol id="svg-twitter" viewBox="0 0 273.4 222.2">
                <path d="M273.4 26.3c-10.1 4.5-20.9 7.5-32.2 8.8 11.6-6.9 20.5-17.9 24.7-31-10.9 6.4-22.9 11.1-35.7 13.6C220 6.8 205.4 0 189.3 0c-31 0-56.1 25.1-56.1 56.1 0 4.4.5 8.7 1.5 12.8C88 66.5 46.7 44.2 19 10.3c-4.8 8.3-7.6 17.9-7.6 28.2 0 19.5 9.9 36.6 25 46.7-9.2-.3-17.8-2.8-25.4-7v.7c0 27.2 19.3 49.8 45 55-4.7 1.3-9.7 2-14.8 2-3.6 0-7.1-.4-10.6-1 7.1 22.3 27.9 38.5 52.4 39-19.2 15-43.4 24-69.7 24-4.5 0-9-.3-13.4-.8 24.8 15.9 54.3 25.2 86 25.2 103.2 0 159.6-85.5 159.6-159.6 0-2.4-.1-4.9-.2-7.3 11.1-8 20.6-17.9 28.1-29.1z"/>
            </symbol>
            <symbol id="svg-circle-play" viewBox="0 0 191.4 191.4">
                <circle fill="none" stroke="#FFF" stroke-width="22" stroke-miterlimit="10" cx="95.7" cy="95.7" r="84.7"/>
                <path d="M87.8 57l46.7 32.6c4.2 3 4.2 9.2 0 12.2l-45.3 31.6c-4.7 3.3-11.1-.1-11.1-5.8V62c0-4.9 5.6-7.9 9.7-5z"/>
            </symbol>
            <symbol id="svg-code" viewBox="0 0 20 20">
                <path d="M5.72 14.75c-.237 0-.475-.083-.665-.252L-.005 10l5.34-4.748c.413-.365 1.045-.33 1.412.083.367.413.33 1.045-.083 1.412L3.004 10l3.38 3.002c.412.367.45 1 .082 1.412-.197.223-.472.336-.747.336zm8.944-.002L20.004 10l-5.06-4.498c-.412-.367-1.044-.33-1.41.083-.367.413-.33 1.045.083 1.412L16.995 10l-3.66 3.252c-.412.367-.45 1-.082 1.412.197.223.472.336.747.336.236 0 .474-.083.664-.252zm-4.678 1.417l2-12c.09-.545-.277-1.06-.822-1.15-.547-.093-1.06.276-1.15.82l-2 12c-.09.546.277 1.06.822 1.152.056.01.11.013.165.013.48 0 .905-.347.986-.835z"/>
            </symbol>
            <symbol id="svg-menu" viewBox="0 0 20 20">
                <path d="M16.4 9H3.6c-.552 0-.6.447-.6 1 0 .553.048 1 .6 1h12.8c.552 0 .6-.447.6-1 0-.553-.048-1-.6-1zm0 4H3.6c-.552 0-.6.447-.6 1 0 .553.048 1 .6 1h12.8c.552 0 .6-.447.6-1 0-.553-.048-1-.6-1zM3.6 7h12.8c.552 0 .6-.447.6-1 0-.553-.048-1-.6-1H3.6c-.552 0-.6.447-.6 1 0 .553.048 1 .6 1z"/>
            </symbol>
            <symbol id="svg-cross" viewBox="0 0 20 20">
                <path d="M14.348 14.85c-.47.468-1.23.468-1.697 0L10 11.82l-2.65 3.028c-.47.47-1.23.47-1.698 0-.47-.47-.47-1.23 0-1.697L8.41 10 5.65 6.85c-.468-.47-.468-1.23 0-1.698.47-.47 1.23-.47 1.698 0L10 8.182l2.65-3.03c.47-.47 1.23-.47 1.698 0 .47.47.47 1.23 0 1.697L11.59 10l2.758 3.15c.47.47.47 1.23 0 1.7z"/>
            </symbol>
            <symbol id="svg-typeface-reg" viewBox="0 0 113.8 77.2">
                <path d="M20.9 0h18.5l20.9 76.1h-8.4l-5.5-19.6H13.9L8.4 76.1H0L20.9 0zm-5.2 49h28.8L33 7.3h-5.7L15.7 49zM107.5 65.9c.2 3.2 2.9 4.4 6.4 4.8l-.3 6.5c-5.8 0-9.8-1.1-13.1-4.4 0 0-9.9 4.4-19.8 4.4-10 0-15.5-5.7-15.5-16.8 0-10.6 5.5-15.2 16.8-16.3l17.3-1.6v-4.7c0-7.7-3.3-10.5-9.9-10.5-7.7 0-20.8 1.4-20.8 1.4l-.3-6.3S80.4 20 89.9 20c12.4 0 17.7 5.7 17.7 17.7v28.2zM82.9 50.3c-6.7.7-9.4 3.9-9.4 9.9 0 6.4 2.8 10.1 8.4 10.1 8.1 0 17.3-3.4 17.3-3.4V48.7l-16.3 1.6z"/>
            </symbol>
            <symbol id="svg-typeface-bold" viewBox="0 0 114.3 76.6">
                <path d="M18.6 0h24.3l18.7 75.4H49.3l-4.1-16.2H16.3l-4.1 16.2H0L18.6 0zm.1 48.4h24.1l-9.2-38.2H28l-9.3 38.2zM109.5 62.4c.2 3.3 1.7 4.6 4.8 5.1l-.3 9.1c-6.7 0-10.6-.9-14.6-4.1 0 0-8.8 4.1-17.7 4.1-10.9 0-16.4-6-16.4-17.5 0-11.7 6.4-15.6 18.1-16.6l14.2-1.2v-4c0-6.1-2.6-7.9-8-7.9-7.4 0-20.7 1.1-20.7 1.1l-.5-8.5s12-2.9 22.1-2.9c13.4 0 18.9 5.6 18.9 18.2v25.1zM84.8 50.9c-5.1.4-7.6 2.9-7.6 7.8s2.1 8 6.7 8c6.3 0 13.6-2.4 13.6-2.4V49.7l-12.7 1.2z"/>
            </symbol>
            <symbol id="svg-typeface-thin" viewBox="0 0 113.3 78">
                <path d="M23.6 0h11.7L59 77h-4l-7.2-23.6H11.1L4 77H0L23.6 0zM12.3 49.6h34.3l-14-45.9h-6.2L12.3 49.6zM105 69.9c.3 3.2 4.4 4.3 8.2 4.6l-.2 3.4c-4.7 0-8.9-1.2-11.3-4.6 0 0-11.2 4.7-22.2 4.7-9 0-14.4-5.4-14.4-16.1 0-9.3 4.4-14.7 15.2-15.8l20.9-2.2v-5.7c0-9.6-4.2-13.4-12.2-13.4s-20.8 1.9-20.8 1.9l-.3-3.7s12.3-2 21.1-2c11.1 0 16.1 5.8 16.1 17.2v31.7zM80.7 49.5c-8.6.9-11.5 4.8-11.5 12.4 0 8 3.7 12.5 10.5 12.5 10.3 0 21.6-4.5 21.6-4.5V47.4l-20.6 2.1z"/>
            </symbol>
        </svg>
        ${this.icon}
    `;
  }
};
__decorateClass([
  n4({ type: String, attribute: "icon-name" })
], BsIcon.prototype, "iconName", 2);
customElements.define("bs-icon", BsIcon);

// src/index.ts
var all = fetch("/__bs_api/servers").then((x2) => x2.json());
var me = fetch("/__bs_api/me").then((x2) => x2.json());
Promise.all([all, me]).then(([servers, me2]) => {
  let next = x`<bs-debug .servers=${servers} .me=${me2}></bs-debug>`;
  let app = document.querySelector("#app");
  if (!app)
    throw new Error("cannot...");
  j(next, app);
}).catch(console.error);
/*! Bundled license information:

@lit/reactive-element/css-tag.js:
  (**
   * @license
   * Copyright 2019 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/reactive-element.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

lit-html/lit-html.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

lit-element/lit-element.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

lit-html/is-server.js:
  (**
   * @license
   * Copyright 2022 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/custom-element.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/property.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/state.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/event-options.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/base.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/query.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/query-all.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/query-async.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/query-assigned-elements.js:
  (**
   * @license
   * Copyright 2021 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/decorators/query-assigned-nodes.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)
*/
