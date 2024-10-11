var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __decorateClass = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc(target, key) : target;
  for (var i3 = decorators.length - 1, decorator; i3 >= 0; i3--)
    if (decorator = decorators[i3])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result)
    __defProp(target, key, result);
  return result;
};

// ../../../node_modules/@lit/reactive-element/css-tag.js
var t = globalThis;
var e = t.ShadowRoot && (void 0 === t.ShadyCSS || t.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype;
var s = Symbol();
var o = /* @__PURE__ */ new WeakMap();
var n = class {
  constructor(t2, e4, o4) {
    if (this._$cssResult$ = true, o4 !== s)
      throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
    this.cssText = t2, this.t = e4;
  }
  get styleSheet() {
    let t2 = this.o;
    const s2 = this.t;
    if (e && void 0 === t2) {
      const e4 = void 0 !== s2 && 1 === s2.length;
      e4 && (t2 = o.get(s2)), void 0 === t2 && ((this.o = t2 = new CSSStyleSheet()).replaceSync(this.cssText), e4 && o.set(s2, t2));
    }
    return t2;
  }
  toString() {
    return this.cssText;
  }
};
var r = (t2) => new n("string" == typeof t2 ? t2 : t2 + "", void 0, s);
var i = (t2, ...e4) => {
  const o4 = 1 === t2.length ? t2[0] : e4.reduce((e5, s2, o5) => e5 + ((t3) => {
    if (true === t3._$cssResult$)
      return t3.cssText;
    if ("number" == typeof t3)
      return t3;
    throw Error("Value passed to 'css' function must be a 'css' function result: " + t3 + ". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.");
  })(s2) + t2[o5 + 1], t2[0]);
  return new n(o4, t2, s);
};
var S = (s2, o4) => {
  if (e)
    s2.adoptedStyleSheets = o4.map((t2) => t2 instanceof CSSStyleSheet ? t2 : t2.styleSheet);
  else
    for (const e4 of o4) {
      const o5 = document.createElement("style"), n5 = t.litNonce;
      void 0 !== n5 && o5.setAttribute("nonce", n5), o5.textContent = e4.cssText, s2.appendChild(o5);
    }
};
var c = e ? (t2) => t2 : (t2) => t2 instanceof CSSStyleSheet ? ((t3) => {
  let e4 = "";
  for (const s2 of t3.cssRules)
    e4 += s2.cssText;
  return r(e4);
})(t2) : t2;

// ../../../node_modules/@lit/reactive-element/reactive-element.js
var { is: i2, defineProperty: e2, getOwnPropertyDescriptor: r2, getOwnPropertyNames: h, getOwnPropertySymbols: o2, getPrototypeOf: n2 } = Object;
var a = globalThis;
var c2 = a.trustedTypes;
var l = c2 ? c2.emptyScript : "";
var p = a.reactiveElementPolyfillSupport;
var d = (t2, s2) => t2;
var u = { toAttribute(t2, s2) {
  switch (s2) {
    case Boolean:
      t2 = t2 ? l : null;
      break;
    case Object:
    case Array:
      t2 = null == t2 ? t2 : JSON.stringify(t2);
  }
  return t2;
}, fromAttribute(t2, s2) {
  let i3 = t2;
  switch (s2) {
    case Boolean:
      i3 = null !== t2;
      break;
    case Number:
      i3 = null === t2 ? null : Number(t2);
      break;
    case Object:
    case Array:
      try {
        i3 = JSON.parse(t2);
      } catch (t3) {
        i3 = null;
      }
  }
  return i3;
} };
var f = (t2, s2) => !i2(t2, s2);
var y = { attribute: true, type: String, converter: u, reflect: false, hasChanged: f };
Symbol.metadata ??= Symbol("metadata"), a.litPropertyMetadata ??= /* @__PURE__ */ new WeakMap();
var b = class extends HTMLElement {
  static addInitializer(t2) {
    this._$Ei(), (this.l ??= []).push(t2);
  }
  static get observedAttributes() {
    return this.finalize(), this._$Eh && [...this._$Eh.keys()];
  }
  static createProperty(t2, s2 = y) {
    if (s2.state && (s2.attribute = false), this._$Ei(), this.elementProperties.set(t2, s2), !s2.noAccessor) {
      const i3 = Symbol(), r4 = this.getPropertyDescriptor(t2, i3, s2);
      void 0 !== r4 && e2(this.prototype, t2, r4);
    }
  }
  static getPropertyDescriptor(t2, s2, i3) {
    const { get: e4, set: h4 } = r2(this.prototype, t2) ?? { get() {
      return this[s2];
    }, set(t3) {
      this[s2] = t3;
    } };
    return { get() {
      return e4?.call(this);
    }, set(s3) {
      const r4 = e4?.call(this);
      h4.call(this, s3), this.requestUpdate(t2, r4, i3);
    }, configurable: true, enumerable: true };
  }
  static getPropertyOptions(t2) {
    return this.elementProperties.get(t2) ?? y;
  }
  static _$Ei() {
    if (this.hasOwnProperty(d("elementProperties")))
      return;
    const t2 = n2(this);
    t2.finalize(), void 0 !== t2.l && (this.l = [...t2.l]), this.elementProperties = new Map(t2.elementProperties);
  }
  static finalize() {
    if (this.hasOwnProperty(d("finalized")))
      return;
    if (this.finalized = true, this._$Ei(), this.hasOwnProperty(d("properties"))) {
      const t3 = this.properties, s2 = [...h(t3), ...o2(t3)];
      for (const i3 of s2)
        this.createProperty(i3, t3[i3]);
    }
    const t2 = this[Symbol.metadata];
    if (null !== t2) {
      const s2 = litPropertyMetadata.get(t2);
      if (void 0 !== s2)
        for (const [t3, i3] of s2)
          this.elementProperties.set(t3, i3);
    }
    this._$Eh = /* @__PURE__ */ new Map();
    for (const [t3, s2] of this.elementProperties) {
      const i3 = this._$Eu(t3, s2);
      void 0 !== i3 && this._$Eh.set(i3, t3);
    }
    this.elementStyles = this.finalizeStyles(this.styles);
  }
  static finalizeStyles(s2) {
    const i3 = [];
    if (Array.isArray(s2)) {
      const e4 = new Set(s2.flat(1 / 0).reverse());
      for (const s3 of e4)
        i3.unshift(c(s3));
    } else
      void 0 !== s2 && i3.push(c(s2));
    return i3;
  }
  static _$Eu(t2, s2) {
    const i3 = s2.attribute;
    return false === i3 ? void 0 : "string" == typeof i3 ? i3 : "string" == typeof t2 ? t2.toLowerCase() : void 0;
  }
  constructor() {
    super(), this._$Ep = void 0, this.isUpdatePending = false, this.hasUpdated = false, this._$Em = null, this._$Ev();
  }
  _$Ev() {
    this._$ES = new Promise((t2) => this.enableUpdating = t2), this._$AL = /* @__PURE__ */ new Map(), this._$E_(), this.requestUpdate(), this.constructor.l?.forEach((t2) => t2(this));
  }
  addController(t2) {
    (this._$EO ??= /* @__PURE__ */ new Set()).add(t2), void 0 !== this.renderRoot && this.isConnected && t2.hostConnected?.();
  }
  removeController(t2) {
    this._$EO?.delete(t2);
  }
  _$E_() {
    const t2 = /* @__PURE__ */ new Map(), s2 = this.constructor.elementProperties;
    for (const i3 of s2.keys())
      this.hasOwnProperty(i3) && (t2.set(i3, this[i3]), delete this[i3]);
    t2.size > 0 && (this._$Ep = t2);
  }
  createRenderRoot() {
    const t2 = this.shadowRoot ?? this.attachShadow(this.constructor.shadowRootOptions);
    return S(t2, this.constructor.elementStyles), t2;
  }
  connectedCallback() {
    this.renderRoot ??= this.createRenderRoot(), this.enableUpdating(true), this._$EO?.forEach((t2) => t2.hostConnected?.());
  }
  enableUpdating(t2) {
  }
  disconnectedCallback() {
    this._$EO?.forEach((t2) => t2.hostDisconnected?.());
  }
  attributeChangedCallback(t2, s2, i3) {
    this._$AK(t2, i3);
  }
  _$EC(t2, s2) {
    const i3 = this.constructor.elementProperties.get(t2), e4 = this.constructor._$Eu(t2, i3);
    if (void 0 !== e4 && true === i3.reflect) {
      const r4 = (void 0 !== i3.converter?.toAttribute ? i3.converter : u).toAttribute(s2, i3.type);
      this._$Em = t2, null == r4 ? this.removeAttribute(e4) : this.setAttribute(e4, r4), this._$Em = null;
    }
  }
  _$AK(t2, s2) {
    const i3 = this.constructor, e4 = i3._$Eh.get(t2);
    if (void 0 !== e4 && this._$Em !== e4) {
      const t3 = i3.getPropertyOptions(e4), r4 = "function" == typeof t3.converter ? { fromAttribute: t3.converter } : void 0 !== t3.converter?.fromAttribute ? t3.converter : u;
      this._$Em = e4, this[e4] = r4.fromAttribute(s2, t3.type), this._$Em = null;
    }
  }
  requestUpdate(t2, s2, i3) {
    if (void 0 !== t2) {
      if (i3 ??= this.constructor.getPropertyOptions(t2), !(i3.hasChanged ?? f)(this[t2], s2))
        return;
      this.P(t2, s2, i3);
    }
    false === this.isUpdatePending && (this._$ES = this._$ET());
  }
  P(t2, s2, i3) {
    this._$AL.has(t2) || this._$AL.set(t2, s2), true === i3.reflect && this._$Em !== t2 && (this._$Ej ??= /* @__PURE__ */ new Set()).add(t2);
  }
  async _$ET() {
    this.isUpdatePending = true;
    try {
      await this._$ES;
    } catch (t3) {
      Promise.reject(t3);
    }
    const t2 = this.scheduleUpdate();
    return null != t2 && await t2, !this.isUpdatePending;
  }
  scheduleUpdate() {
    return this.performUpdate();
  }
  performUpdate() {
    if (!this.isUpdatePending)
      return;
    if (!this.hasUpdated) {
      if (this.renderRoot ??= this.createRenderRoot(), this._$Ep) {
        for (const [t4, s3] of this._$Ep)
          this[t4] = s3;
        this._$Ep = void 0;
      }
      const t3 = this.constructor.elementProperties;
      if (t3.size > 0)
        for (const [s3, i3] of t3)
          true !== i3.wrapped || this._$AL.has(s3) || void 0 === this[s3] || this.P(s3, this[s3], i3);
    }
    let t2 = false;
    const s2 = this._$AL;
    try {
      t2 = this.shouldUpdate(s2), t2 ? (this.willUpdate(s2), this._$EO?.forEach((t3) => t3.hostUpdate?.()), this.update(s2)) : this._$EU();
    } catch (s3) {
      throw t2 = false, this._$EU(), s3;
    }
    t2 && this._$AE(s2);
  }
  willUpdate(t2) {
  }
  _$AE(t2) {
    this._$EO?.forEach((t3) => t3.hostUpdated?.()), this.hasUpdated || (this.hasUpdated = true, this.firstUpdated(t2)), this.updated(t2);
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
  shouldUpdate(t2) {
    return true;
  }
  update(t2) {
    this._$Ej &&= this._$Ej.forEach((t3) => this._$EC(t3, this[t3])), this._$EU();
  }
  updated(t2) {
  }
  firstUpdated(t2) {
  }
};
b.elementStyles = [], b.shadowRootOptions = { mode: "open" }, b[d("elementProperties")] = /* @__PURE__ */ new Map(), b[d("finalized")] = /* @__PURE__ */ new Map(), p?.({ ReactiveElement: b }), (a.reactiveElementVersions ??= []).push("2.0.4");

// ../../../node_modules/lit-html/lit-html.js
var n3 = globalThis;
var c3 = n3.trustedTypes;
var h2 = c3 ? c3.createPolicy("lit-html", { createHTML: (t2) => t2 }) : void 0;
var f2 = "$lit$";
var v = `lit$${Math.random().toFixed(9).slice(2)}$`;
var m = "?" + v;
var _ = `<${m}>`;
var w = document;
var lt = () => w.createComment("");
var st = (t2) => null === t2 || "object" != typeof t2 && "function" != typeof t2;
var g = Array.isArray;
var $ = (t2) => g(t2) || "function" == typeof t2?.[Symbol.iterator];
var x = "[ 	\n\f\r]";
var T = /<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g;
var E = /-->/g;
var k = />/g;
var O = RegExp(`>|${x}(?:([^\\s"'>=/]+)(${x}*=${x}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`, "g");
var S2 = /'/g;
var j = /"/g;
var M = /^(?:script|style|textarea|title)$/i;
var P = (t2) => (i3, ...s2) => ({ _$litType$: t2, strings: i3, values: s2 });
var ke = P(1);
var Oe = P(2);
var Se = P(3);
var R = Symbol.for("lit-noChange");
var D = Symbol.for("lit-nothing");
var V = /* @__PURE__ */ new WeakMap();
var I = w.createTreeWalker(w, 129);
function N(t2, i3) {
  if (!g(t2) || !t2.hasOwnProperty("raw"))
    throw Error("invalid template strings array");
  return void 0 !== h2 ? h2.createHTML(i3) : i3;
}
var U = (t2, i3) => {
  const s2 = t2.length - 1, e4 = [];
  let h4, o4 = 2 === i3 ? "<svg>" : 3 === i3 ? "<math>" : "", n5 = T;
  for (let i4 = 0; i4 < s2; i4++) {
    const s3 = t2[i4];
    let r4, l2, c4 = -1, a2 = 0;
    for (; a2 < s3.length && (n5.lastIndex = a2, l2 = n5.exec(s3), null !== l2); )
      a2 = n5.lastIndex, n5 === T ? "!--" === l2[1] ? n5 = E : void 0 !== l2[1] ? n5 = k : void 0 !== l2[2] ? (M.test(l2[2]) && (h4 = RegExp("</" + l2[2], "g")), n5 = O) : void 0 !== l2[3] && (n5 = O) : n5 === O ? ">" === l2[0] ? (n5 = h4 ?? T, c4 = -1) : void 0 === l2[1] ? c4 = -2 : (c4 = n5.lastIndex - l2[2].length, r4 = l2[1], n5 = void 0 === l2[3] ? O : '"' === l2[3] ? j : S2) : n5 === j || n5 === S2 ? n5 = O : n5 === E || n5 === k ? n5 = T : (n5 = O, h4 = void 0);
    const u2 = n5 === O && t2[i4 + 1].startsWith("/>") ? " " : "";
    o4 += n5 === T ? s3 + _ : c4 >= 0 ? (e4.push(r4), s3.slice(0, c4) + f2 + s3.slice(c4) + v + u2) : s3 + v + (-2 === c4 ? i4 : u2);
  }
  return [N(t2, o4 + (t2[s2] || "<?>") + (2 === i3 ? "</svg>" : 3 === i3 ? "</math>" : "")), e4];
};
var B = class _B {
  constructor({ strings: t2, _$litType$: i3 }, s2) {
    let e4;
    this.parts = [];
    let h4 = 0, o4 = 0;
    const n5 = t2.length - 1, r4 = this.parts, [l2, a2] = U(t2, i3);
    if (this.el = _B.createElement(l2, s2), I.currentNode = this.el.content, 2 === i3 || 3 === i3) {
      const t3 = this.el.content.firstChild;
      t3.replaceWith(...t3.childNodes);
    }
    for (; null !== (e4 = I.nextNode()) && r4.length < n5; ) {
      if (1 === e4.nodeType) {
        if (e4.hasAttributes())
          for (const t3 of e4.getAttributeNames())
            if (t3.endsWith(f2)) {
              const i4 = a2[o4++], s3 = e4.getAttribute(t3).split(v), n6 = /([.?@])?(.*)/.exec(i4);
              r4.push({ type: 1, index: h4, name: n6[2], strings: s3, ctor: "." === n6[1] ? Y : "?" === n6[1] ? Z : "@" === n6[1] ? q : G }), e4.removeAttribute(t3);
            } else
              t3.startsWith(v) && (r4.push({ type: 6, index: h4 }), e4.removeAttribute(t3));
        if (M.test(e4.tagName)) {
          const t3 = e4.textContent.split(v), i4 = t3.length - 1;
          if (i4 > 0) {
            e4.textContent = c3 ? c3.emptyScript : "";
            for (let s3 = 0; s3 < i4; s3++)
              e4.append(t3[s3], lt()), I.nextNode(), r4.push({ type: 2, index: ++h4 });
            e4.append(t3[i4], lt());
          }
        }
      } else if (8 === e4.nodeType)
        if (e4.data === m)
          r4.push({ type: 2, index: h4 });
        else {
          let t3 = -1;
          for (; -1 !== (t3 = e4.data.indexOf(v, t3 + 1)); )
            r4.push({ type: 7, index: h4 }), t3 += v.length - 1;
        }
      h4++;
    }
  }
  static createElement(t2, i3) {
    const s2 = w.createElement("template");
    return s2.innerHTML = t2, s2;
  }
};
function z(t2, i3, s2 = t2, e4) {
  if (i3 === R)
    return i3;
  let h4 = void 0 !== e4 ? s2.o?.[e4] : s2.l;
  const o4 = st(i3) ? void 0 : i3._$litDirective$;
  return h4?.constructor !== o4 && (h4?._$AO?.(false), void 0 === o4 ? h4 = void 0 : (h4 = new o4(t2), h4._$AT(t2, s2, e4)), void 0 !== e4 ? (s2.o ??= [])[e4] = h4 : s2.l = h4), void 0 !== h4 && (i3 = z(t2, h4._$AS(t2, i3.values), h4, e4)), i3;
}
var F = class {
  constructor(t2, i3) {
    this._$AV = [], this._$AN = void 0, this._$AD = t2, this._$AM = i3;
  }
  get parentNode() {
    return this._$AM.parentNode;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  u(t2) {
    const { el: { content: i3 }, parts: s2 } = this._$AD, e4 = (t2?.creationScope ?? w).importNode(i3, true);
    I.currentNode = e4;
    let h4 = I.nextNode(), o4 = 0, n5 = 0, r4 = s2[0];
    for (; void 0 !== r4; ) {
      if (o4 === r4.index) {
        let i4;
        2 === r4.type ? i4 = new et(h4, h4.nextSibling, this, t2) : 1 === r4.type ? i4 = new r4.ctor(h4, r4.name, r4.strings, this, t2) : 6 === r4.type && (i4 = new K(h4, this, t2)), this._$AV.push(i4), r4 = s2[++n5];
      }
      o4 !== r4?.index && (h4 = I.nextNode(), o4++);
    }
    return I.currentNode = w, e4;
  }
  p(t2) {
    let i3 = 0;
    for (const s2 of this._$AV)
      void 0 !== s2 && (void 0 !== s2.strings ? (s2._$AI(t2, s2, i3), i3 += s2.strings.length - 2) : s2._$AI(t2[i3])), i3++;
  }
};
var et = class _et {
  get _$AU() {
    return this._$AM?._$AU ?? this.v;
  }
  constructor(t2, i3, s2, e4) {
    this.type = 2, this._$AH = D, this._$AN = void 0, this._$AA = t2, this._$AB = i3, this._$AM = s2, this.options = e4, this.v = e4?.isConnected ?? true;
  }
  get parentNode() {
    let t2 = this._$AA.parentNode;
    const i3 = this._$AM;
    return void 0 !== i3 && 11 === t2?.nodeType && (t2 = i3.parentNode), t2;
  }
  get startNode() {
    return this._$AA;
  }
  get endNode() {
    return this._$AB;
  }
  _$AI(t2, i3 = this) {
    t2 = z(this, t2, i3), st(t2) ? t2 === D || null == t2 || "" === t2 ? (this._$AH !== D && this._$AR(), this._$AH = D) : t2 !== this._$AH && t2 !== R && this._(t2) : void 0 !== t2._$litType$ ? this.$(t2) : void 0 !== t2.nodeType ? this.T(t2) : $(t2) ? this.k(t2) : this._(t2);
  }
  O(t2) {
    return this._$AA.parentNode.insertBefore(t2, this._$AB);
  }
  T(t2) {
    this._$AH !== t2 && (this._$AR(), this._$AH = this.O(t2));
  }
  _(t2) {
    this._$AH !== D && st(this._$AH) ? this._$AA.nextSibling.data = t2 : this.T(w.createTextNode(t2)), this._$AH = t2;
  }
  $(t2) {
    const { values: i3, _$litType$: s2 } = t2, e4 = "number" == typeof s2 ? this._$AC(t2) : (void 0 === s2.el && (s2.el = B.createElement(N(s2.h, s2.h[0]), this.options)), s2);
    if (this._$AH?._$AD === e4)
      this._$AH.p(i3);
    else {
      const t3 = new F(e4, this), s3 = t3.u(this.options);
      t3.p(i3), this.T(s3), this._$AH = t3;
    }
  }
  _$AC(t2) {
    let i3 = V.get(t2.strings);
    return void 0 === i3 && V.set(t2.strings, i3 = new B(t2)), i3;
  }
  k(t2) {
    g(this._$AH) || (this._$AH = [], this._$AR());
    const i3 = this._$AH;
    let s2, e4 = 0;
    for (const h4 of t2)
      e4 === i3.length ? i3.push(s2 = new _et(this.O(lt()), this.O(lt()), this, this.options)) : s2 = i3[e4], s2._$AI(h4), e4++;
    e4 < i3.length && (this._$AR(s2 && s2._$AB.nextSibling, e4), i3.length = e4);
  }
  _$AR(t2 = this._$AA.nextSibling, i3) {
    for (this._$AP?.(false, true, i3); t2 && t2 !== this._$AB; ) {
      const i4 = t2.nextSibling;
      t2.remove(), t2 = i4;
    }
  }
  setConnected(t2) {
    void 0 === this._$AM && (this.v = t2, this._$AP?.(t2));
  }
};
var G = class {
  get tagName() {
    return this.element.tagName;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  constructor(t2, i3, s2, e4, h4) {
    this.type = 1, this._$AH = D, this._$AN = void 0, this.element = t2, this.name = i3, this._$AM = e4, this.options = h4, s2.length > 2 || "" !== s2[0] || "" !== s2[1] ? (this._$AH = Array(s2.length - 1).fill(new String()), this.strings = s2) : this._$AH = D;
  }
  _$AI(t2, i3 = this, s2, e4) {
    const h4 = this.strings;
    let o4 = false;
    if (void 0 === h4)
      t2 = z(this, t2, i3, 0), o4 = !st(t2) || t2 !== this._$AH && t2 !== R, o4 && (this._$AH = t2);
    else {
      const e5 = t2;
      let n5, r4;
      for (t2 = h4[0], n5 = 0; n5 < h4.length - 1; n5++)
        r4 = z(this, e5[s2 + n5], i3, n5), r4 === R && (r4 = this._$AH[n5]), o4 ||= !st(r4) || r4 !== this._$AH[n5], r4 === D ? t2 = D : t2 !== D && (t2 += (r4 ?? "") + h4[n5 + 1]), this._$AH[n5] = r4;
    }
    o4 && !e4 && this.j(t2);
  }
  j(t2) {
    t2 === D ? this.element.removeAttribute(this.name) : this.element.setAttribute(this.name, t2 ?? "");
  }
};
var Y = class extends G {
  constructor() {
    super(...arguments), this.type = 3;
  }
  j(t2) {
    this.element[this.name] = t2 === D ? void 0 : t2;
  }
};
var Z = class extends G {
  constructor() {
    super(...arguments), this.type = 4;
  }
  j(t2) {
    this.element.toggleAttribute(this.name, !!t2 && t2 !== D);
  }
};
var q = class extends G {
  constructor(t2, i3, s2, e4, h4) {
    super(t2, i3, s2, e4, h4), this.type = 5;
  }
  _$AI(t2, i3 = this) {
    if ((t2 = z(this, t2, i3, 0) ?? D) === R)
      return;
    const s2 = this._$AH, e4 = t2 === D && s2 !== D || t2.capture !== s2.capture || t2.once !== s2.once || t2.passive !== s2.passive, h4 = t2 !== D && (s2 === D || e4);
    e4 && this.element.removeEventListener(this.name, this, s2), h4 && this.element.addEventListener(this.name, this, t2), this._$AH = t2;
  }
  handleEvent(t2) {
    "function" == typeof this._$AH ? this._$AH.call(this.options?.host ?? this.element, t2) : this._$AH.handleEvent(t2);
  }
};
var K = class {
  constructor(t2, i3, s2) {
    this.element = t2, this.type = 6, this._$AN = void 0, this._$AM = i3, this.options = s2;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AI(t2) {
    z(this, t2);
  }
};
var Re = n3.litHtmlPolyfillSupport;
Re?.(B, et), (n3.litHtmlVersions ??= []).push("3.2.0");
var Q = (t2, i3, s2) => {
  const e4 = s2?.renderBefore ?? i3;
  let h4 = e4._$litPart$;
  if (void 0 === h4) {
    const t3 = s2?.renderBefore ?? null;
    e4._$litPart$ = h4 = new et(i3.insertBefore(lt(), t3), t3, void 0, s2 ?? {});
  }
  return h4._$AI(t2), h4;
};

// ../../../node_modules/lit-element/lit-element.js
var h3 = class extends b {
  constructor() {
    super(...arguments), this.renderOptions = { host: this }, this.o = void 0;
  }
  createRenderRoot() {
    const t2 = super.createRenderRoot();
    return this.renderOptions.renderBefore ??= t2.firstChild, t2;
  }
  update(t2) {
    const e4 = this.render();
    this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(t2), this.o = Q(e4, this.renderRoot, this.renderOptions);
  }
  connectedCallback() {
    super.connectedCallback(), this.o?.setConnected(true);
  }
  disconnectedCallback() {
    super.disconnectedCallback(), this.o?.setConnected(false);
  }
  render() {
    return R;
  }
};
h3._$litElement$ = true, h3["finalized"] = true, globalThis.litElementHydrateSupport?.({ LitElement: h3 });
var f3 = globalThis.litElementPolyfillSupport;
f3?.({ LitElement: h3 });
(globalThis.litElementVersions ??= []).push("4.1.0");

// ../../../node_modules/@lit/reactive-element/decorators/property.js
var o3 = { attribute: true, type: String, converter: u, reflect: false, hasChanged: f };
var r3 = (t2 = o3, e4, r4) => {
  const { kind: n5, metadata: i3 } = r4;
  let s2 = globalThis.litPropertyMetadata.get(i3);
  if (void 0 === s2 && globalThis.litPropertyMetadata.set(i3, s2 = /* @__PURE__ */ new Map()), s2.set(r4.name, t2), "accessor" === n5) {
    const { name: o4 } = r4;
    return { set(r5) {
      const n6 = e4.get.call(this);
      e4.set.call(this, r5), this.requestUpdate(o4, n6, t2);
    }, init(e5) {
      return void 0 !== e5 && this.P(o4, void 0, t2), e5;
    } };
  }
  if ("setter" === n5) {
    const { name: o4 } = r4;
    return function(r5) {
      const n6 = this[o4];
      e4.call(this, r5), this.requestUpdate(o4, n6, t2);
    };
  }
  throw Error("Unsupported decorator location: " + n5);
};
function n4(t2) {
  return (e4, o4) => "object" == typeof o4 ? r3(t2, e4, o4) : ((t3, e5, o5) => {
    const r4 = e5.hasOwnProperty(o5);
    return e5.constructor.createProperty(o5, r4 ? { ...t3, wrapped: true } : t3), r4 ? Object.getOwnPropertyDescriptor(e5, o5) : void 0;
  })(t2, e4, o4);
}

// src/components/bs-debug.ts
var BsDebug = class extends h3 {
  constructor() {
    super(...arguments);
    this.servers = { servers: [] };
    this.me = { routes: [], id: "" };
  }
  get otherServers() {
    return this.servers.servers.filter((server) => server.id !== this.me.id);
  }
  render() {
    return ke`
        <bs-header></bs-header>
        <bs-server-detail .server=${this.me}></bs-server-detail>
        ${this.otherServers.length > 0 ? ke`
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
var BsServerList = class extends h3 {
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
    return ke`
        ${this.servers.map((server) => {
      const display_addr = "http://" + server.socket_addr;
      let url = new URL(display_addr);
      let bs_url = new URL("./__bslive", display_addr);
      return ke`
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
var BsServerDetail = class extends h3 {
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
    return ke`
       <pre><code>${JSON.stringify(this.server, null, 2)}</code></pre>
    `;
  }
};
__decorateClass([
  n4({ type: Object })
], BsServerDetail.prototype, "server", 2);
customElements.define("bs-server-detail", BsServerDetail);

// src/components/bs-server-identity.ts
var BsServerIdentity = class extends h3 {
  static {
    this.styles = [base];
  }
  render() {
    switch (this.identity.kind) {
      case "Named":
      case "Both": {
        return ke`<p><strong>[named] ${this.identity.payload.name}</strong></p>`;
      }
      default:
        return ke`<p><strong>[unnamed]</strong></p>`;
    }
  }
};
__decorateClass([
  n4({ type: Object })
], BsServerIdentity.prototype, "identity", 2);
customElements.define("bs-server-identity", BsServerIdentity);

// src/components/bs-header.ts
var BsHeader = class extends h3 {
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
    return ke`
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
var BsIcon = class extends h3 {
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
        return ke`<svg class="svg-icon" part="svg"><use xlink:href="#svg-logo"></use></svg>`;
      case "wordmark":
        return ke`<svg class="svg-icon" part="svg"><use xlink:href="#svg-wordmark"></use></svg>`;
      default:
        return `unknown`;
    }
  }
  render() {
    return ke`
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
  let next = ke`
        <bs-debug .servers=${servers} .me=${me2}></bs-debug>`;
  let app = document.querySelector("#app");
  if (!app)
    throw new Error("cannot...");
  Q(next, app);
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
