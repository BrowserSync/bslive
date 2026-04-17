var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __decorateClass = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc(target, key) : target;
  for (var i6 = decorators.length - 1, decorator; i6 >= 0; i6--)
    if (decorator = decorators[i6])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp(target, key, result);
  return result;
};

// ../node_modules/@lit/reactive-element/css-tag.js
var t = globalThis;
var e = t.ShadowRoot && (void 0 === t.ShadyCSS || t.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype;
var s = /* @__PURE__ */ Symbol();
var o = /* @__PURE__ */ new WeakMap();
var n = class {
  constructor(t6, e7, o8) {
    if (this._$cssResult$ = true, o8 !== s) throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
    this.cssText = t6, this.t = e7;
  }
  get styleSheet() {
    let t6 = this.o;
    const s5 = this.t;
    if (e && void 0 === t6) {
      const e7 = void 0 !== s5 && 1 === s5.length;
      e7 && (t6 = o.get(s5)), void 0 === t6 && ((this.o = t6 = new CSSStyleSheet()).replaceSync(this.cssText), e7 && o.set(s5, t6));
    }
    return t6;
  }
  toString() {
    return this.cssText;
  }
};
var r = (t6) => new n("string" == typeof t6 ? t6 : t6 + "", void 0, s);
var i = (t6, ...e7) => {
  const o8 = 1 === t6.length ? t6[0] : e7.reduce((e8, s5, o9) => e8 + ((t7) => {
    if (true === t7._$cssResult$) return t7.cssText;
    if ("number" == typeof t7) return t7;
    throw Error("Value passed to 'css' function must be a 'css' function result: " + t7 + ". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.");
  })(s5) + t6[o9 + 1], t6[0]);
  return new n(o8, t6, s);
};
var S = (s5, o8) => {
  if (e) s5.adoptedStyleSheets = o8.map((t6) => t6 instanceof CSSStyleSheet ? t6 : t6.styleSheet);
  else for (const e7 of o8) {
    const o9 = document.createElement("style"), n7 = t.litNonce;
    void 0 !== n7 && o9.setAttribute("nonce", n7), o9.textContent = e7.cssText, s5.appendChild(o9);
  }
};
var c = e ? (t6) => t6 : (t6) => t6 instanceof CSSStyleSheet ? ((t7) => {
  let e7 = "";
  for (const s5 of t7.cssRules) e7 += s5.cssText;
  return r(e7);
})(t6) : t6;

// ../node_modules/@lit/reactive-element/reactive-element.js
var { is: i2, defineProperty: e2, getOwnPropertyDescriptor: h, getOwnPropertyNames: r2, getOwnPropertySymbols: o2, getPrototypeOf: n2 } = Object;
var a = globalThis;
var c2 = a.trustedTypes;
var l = c2 ? c2.emptyScript : "";
var p = a.reactiveElementPolyfillSupport;
var d = (t6, s5) => t6;
var u = { toAttribute(t6, s5) {
  switch (s5) {
    case Boolean:
      t6 = t6 ? l : null;
      break;
    case Object:
    case Array:
      t6 = null == t6 ? t6 : JSON.stringify(t6);
  }
  return t6;
}, fromAttribute(t6, s5) {
  let i6 = t6;
  switch (s5) {
    case Boolean:
      i6 = null !== t6;
      break;
    case Number:
      i6 = null === t6 ? null : Number(t6);
      break;
    case Object:
    case Array:
      try {
        i6 = JSON.parse(t6);
      } catch (t7) {
        i6 = null;
      }
  }
  return i6;
} };
var f = (t6, s5) => !i2(t6, s5);
var b = { attribute: true, type: String, converter: u, reflect: false, useDefault: false, hasChanged: f };
Symbol.metadata ??= /* @__PURE__ */ Symbol("metadata"), a.litPropertyMetadata ??= /* @__PURE__ */ new WeakMap();
var y = class extends HTMLElement {
  static addInitializer(t6) {
    this._$Ei(), (this.l ??= []).push(t6);
  }
  static get observedAttributes() {
    return this.finalize(), this._$Eh && [...this._$Eh.keys()];
  }
  static createProperty(t6, s5 = b) {
    if (s5.state && (s5.attribute = false), this._$Ei(), this.prototype.hasOwnProperty(t6) && ((s5 = Object.create(s5)).wrapped = true), this.elementProperties.set(t6, s5), !s5.noAccessor) {
      const i6 = /* @__PURE__ */ Symbol(), h5 = this.getPropertyDescriptor(t6, i6, s5);
      void 0 !== h5 && e2(this.prototype, t6, h5);
    }
  }
  static getPropertyDescriptor(t6, s5, i6) {
    const { get: e7, set: r7 } = h(this.prototype, t6) ?? { get() {
      return this[s5];
    }, set(t7) {
      this[s5] = t7;
    } };
    return { get: e7, set(s6) {
      const h5 = e7?.call(this);
      r7?.call(this, s6), this.requestUpdate(t6, h5, i6);
    }, configurable: true, enumerable: true };
  }
  static getPropertyOptions(t6) {
    return this.elementProperties.get(t6) ?? b;
  }
  static _$Ei() {
    if (this.hasOwnProperty(d("elementProperties"))) return;
    const t6 = n2(this);
    t6.finalize(), void 0 !== t6.l && (this.l = [...t6.l]), this.elementProperties = new Map(t6.elementProperties);
  }
  static finalize() {
    if (this.hasOwnProperty(d("finalized"))) return;
    if (this.finalized = true, this._$Ei(), this.hasOwnProperty(d("properties"))) {
      const t7 = this.properties, s5 = [...r2(t7), ...o2(t7)];
      for (const i6 of s5) this.createProperty(i6, t7[i6]);
    }
    const t6 = this[Symbol.metadata];
    if (null !== t6) {
      const s5 = litPropertyMetadata.get(t6);
      if (void 0 !== s5) for (const [t7, i6] of s5) this.elementProperties.set(t7, i6);
    }
    this._$Eh = /* @__PURE__ */ new Map();
    for (const [t7, s5] of this.elementProperties) {
      const i6 = this._$Eu(t7, s5);
      void 0 !== i6 && this._$Eh.set(i6, t7);
    }
    this.elementStyles = this.finalizeStyles(this.styles);
  }
  static finalizeStyles(s5) {
    const i6 = [];
    if (Array.isArray(s5)) {
      const e7 = new Set(s5.flat(1 / 0).reverse());
      for (const s6 of e7) i6.unshift(c(s6));
    } else void 0 !== s5 && i6.push(c(s5));
    return i6;
  }
  static _$Eu(t6, s5) {
    const i6 = s5.attribute;
    return false === i6 ? void 0 : "string" == typeof i6 ? i6 : "string" == typeof t6 ? t6.toLowerCase() : void 0;
  }
  constructor() {
    super(), this._$Ep = void 0, this.isUpdatePending = false, this.hasUpdated = false, this._$Em = null, this._$Ev();
  }
  _$Ev() {
    this._$ES = new Promise((t6) => this.enableUpdating = t6), this._$AL = /* @__PURE__ */ new Map(), this._$E_(), this.requestUpdate(), this.constructor.l?.forEach((t6) => t6(this));
  }
  addController(t6) {
    (this._$EO ??= /* @__PURE__ */ new Set()).add(t6), void 0 !== this.renderRoot && this.isConnected && t6.hostConnected?.();
  }
  removeController(t6) {
    this._$EO?.delete(t6);
  }
  _$E_() {
    const t6 = /* @__PURE__ */ new Map(), s5 = this.constructor.elementProperties;
    for (const i6 of s5.keys()) this.hasOwnProperty(i6) && (t6.set(i6, this[i6]), delete this[i6]);
    t6.size > 0 && (this._$Ep = t6);
  }
  createRenderRoot() {
    const t6 = this.shadowRoot ?? this.attachShadow(this.constructor.shadowRootOptions);
    return S(t6, this.constructor.elementStyles), t6;
  }
  connectedCallback() {
    this.renderRoot ??= this.createRenderRoot(), this.enableUpdating(true), this._$EO?.forEach((t6) => t6.hostConnected?.());
  }
  enableUpdating(t6) {
  }
  disconnectedCallback() {
    this._$EO?.forEach((t6) => t6.hostDisconnected?.());
  }
  attributeChangedCallback(t6, s5, i6) {
    this._$AK(t6, i6);
  }
  _$ET(t6, s5) {
    const i6 = this.constructor.elementProperties.get(t6), e7 = this.constructor._$Eu(t6, i6);
    if (void 0 !== e7 && true === i6.reflect) {
      const h5 = (void 0 !== i6.converter?.toAttribute ? i6.converter : u).toAttribute(s5, i6.type);
      this._$Em = t6, null == h5 ? this.removeAttribute(e7) : this.setAttribute(e7, h5), this._$Em = null;
    }
  }
  _$AK(t6, s5) {
    const i6 = this.constructor, e7 = i6._$Eh.get(t6);
    if (void 0 !== e7 && this._$Em !== e7) {
      const t7 = i6.getPropertyOptions(e7), h5 = "function" == typeof t7.converter ? { fromAttribute: t7.converter } : void 0 !== t7.converter?.fromAttribute ? t7.converter : u;
      this._$Em = e7;
      const r7 = h5.fromAttribute(s5, t7.type);
      this[e7] = r7 ?? this._$Ej?.get(e7) ?? r7, this._$Em = null;
    }
  }
  requestUpdate(t6, s5, i6, e7 = false, h5) {
    if (void 0 !== t6) {
      const r7 = this.constructor;
      if (false === e7 && (h5 = this[t6]), i6 ??= r7.getPropertyOptions(t6), !((i6.hasChanged ?? f)(h5, s5) || i6.useDefault && i6.reflect && h5 === this._$Ej?.get(t6) && !this.hasAttribute(r7._$Eu(t6, i6)))) return;
      this.C(t6, s5, i6);
    }
    false === this.isUpdatePending && (this._$ES = this._$EP());
  }
  C(t6, s5, { useDefault: i6, reflect: e7, wrapped: h5 }, r7) {
    i6 && !(this._$Ej ??= /* @__PURE__ */ new Map()).has(t6) && (this._$Ej.set(t6, r7 ?? s5 ?? this[t6]), true !== h5 || void 0 !== r7) || (this._$AL.has(t6) || (this.hasUpdated || i6 || (s5 = void 0), this._$AL.set(t6, s5)), true === e7 && this._$Em !== t6 && (this._$Eq ??= /* @__PURE__ */ new Set()).add(t6));
  }
  async _$EP() {
    this.isUpdatePending = true;
    try {
      await this._$ES;
    } catch (t7) {
      Promise.reject(t7);
    }
    const t6 = this.scheduleUpdate();
    return null != t6 && await t6, !this.isUpdatePending;
  }
  scheduleUpdate() {
    return this.performUpdate();
  }
  performUpdate() {
    if (!this.isUpdatePending) return;
    if (!this.hasUpdated) {
      if (this.renderRoot ??= this.createRenderRoot(), this._$Ep) {
        for (const [t8, s6] of this._$Ep) this[t8] = s6;
        this._$Ep = void 0;
      }
      const t7 = this.constructor.elementProperties;
      if (t7.size > 0) for (const [s6, i6] of t7) {
        const { wrapped: t8 } = i6, e7 = this[s6];
        true !== t8 || this._$AL.has(s6) || void 0 === e7 || this.C(s6, void 0, i6, e7);
      }
    }
    let t6 = false;
    const s5 = this._$AL;
    try {
      t6 = this.shouldUpdate(s5), t6 ? (this.willUpdate(s5), this._$EO?.forEach((t7) => t7.hostUpdate?.()), this.update(s5)) : this._$EM();
    } catch (s6) {
      throw t6 = false, this._$EM(), s6;
    }
    t6 && this._$AE(s5);
  }
  willUpdate(t6) {
  }
  _$AE(t6) {
    this._$EO?.forEach((t7) => t7.hostUpdated?.()), this.hasUpdated || (this.hasUpdated = true, this.firstUpdated(t6)), this.updated(t6);
  }
  _$EM() {
    this._$AL = /* @__PURE__ */ new Map(), this.isUpdatePending = false;
  }
  get updateComplete() {
    return this.getUpdateComplete();
  }
  getUpdateComplete() {
    return this._$ES;
  }
  shouldUpdate(t6) {
    return true;
  }
  update(t6) {
    this._$Eq &&= this._$Eq.forEach((t7) => this._$ET(t7, this[t7])), this._$EM();
  }
  updated(t6) {
  }
  firstUpdated(t6) {
  }
};
y.elementStyles = [], y.shadowRootOptions = { mode: "open" }, y[d("elementProperties")] = /* @__PURE__ */ new Map(), y[d("finalized")] = /* @__PURE__ */ new Map(), p?.({ ReactiveElement: y }), (a.reactiveElementVersions ??= []).push("2.1.2");

// ../node_modules/lit-html/lit-html.js
var t2 = globalThis;
var i3 = (t6) => t6;
var s2 = t2.trustedTypes;
var e3 = s2 ? s2.createPolicy("lit-html", { createHTML: (t6) => t6 }) : void 0;
var h2 = "$lit$";
var o3 = `lit$${Math.random().toFixed(9).slice(2)}$`;
var n3 = "?" + o3;
var r3 = `<${n3}>`;
var l2 = document;
var c3 = () => l2.createComment("");
var a2 = (t6) => null === t6 || "object" != typeof t6 && "function" != typeof t6;
var u2 = Array.isArray;
var d2 = (t6) => u2(t6) || "function" == typeof t6?.[Symbol.iterator];
var f2 = "[ 	\n\f\r]";
var v = /<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g;
var _ = /-->/g;
var m = />/g;
var p2 = RegExp(`>|${f2}(?:([^\\s"'>=/]+)(${f2}*=${f2}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`, "g");
var g = /'/g;
var $ = /"/g;
var y2 = /^(?:script|style|textarea|title)$/i;
var x = (t6) => (i6, ...s5) => ({ _$litType$: t6, strings: i6, values: s5 });
var b2 = x(1);
var w = x(2);
var T = x(3);
var E = /* @__PURE__ */ Symbol.for("lit-noChange");
var A = /* @__PURE__ */ Symbol.for("lit-nothing");
var C = /* @__PURE__ */ new WeakMap();
var P = l2.createTreeWalker(l2, 129);
function V(t6, i6) {
  if (!u2(t6) || !t6.hasOwnProperty("raw")) throw Error("invalid template strings array");
  return void 0 !== e3 ? e3.createHTML(i6) : i6;
}
var N = (t6, i6) => {
  const s5 = t6.length - 1, e7 = [];
  let n7, l3 = 2 === i6 ? "<svg>" : 3 === i6 ? "<math>" : "", c5 = v;
  for (let i7 = 0; i7 < s5; i7++) {
    const s6 = t6[i7];
    let a3, u3, d3 = -1, f4 = 0;
    for (; f4 < s6.length && (c5.lastIndex = f4, u3 = c5.exec(s6), null !== u3); ) f4 = c5.lastIndex, c5 === v ? "!--" === u3[1] ? c5 = _ : void 0 !== u3[1] ? c5 = m : void 0 !== u3[2] ? (y2.test(u3[2]) && (n7 = RegExp("</" + u3[2], "g")), c5 = p2) : void 0 !== u3[3] && (c5 = p2) : c5 === p2 ? ">" === u3[0] ? (c5 = n7 ?? v, d3 = -1) : void 0 === u3[1] ? d3 = -2 : (d3 = c5.lastIndex - u3[2].length, a3 = u3[1], c5 = void 0 === u3[3] ? p2 : '"' === u3[3] ? $ : g) : c5 === $ || c5 === g ? c5 = p2 : c5 === _ || c5 === m ? c5 = v : (c5 = p2, n7 = void 0);
    const x2 = c5 === p2 && t6[i7 + 1].startsWith("/>") ? " " : "";
    l3 += c5 === v ? s6 + r3 : d3 >= 0 ? (e7.push(a3), s6.slice(0, d3) + h2 + s6.slice(d3) + o3 + x2) : s6 + o3 + (-2 === d3 ? i7 : x2);
  }
  return [V(t6, l3 + (t6[s5] || "<?>") + (2 === i6 ? "</svg>" : 3 === i6 ? "</math>" : "")), e7];
};
var S2 = class _S {
  constructor({ strings: t6, _$litType$: i6 }, e7) {
    let r7;
    this.parts = [];
    let l3 = 0, a3 = 0;
    const u3 = t6.length - 1, d3 = this.parts, [f4, v2] = N(t6, i6);
    if (this.el = _S.createElement(f4, e7), P.currentNode = this.el.content, 2 === i6 || 3 === i6) {
      const t7 = this.el.content.firstChild;
      t7.replaceWith(...t7.childNodes);
    }
    for (; null !== (r7 = P.nextNode()) && d3.length < u3; ) {
      if (1 === r7.nodeType) {
        if (r7.hasAttributes()) for (const t7 of r7.getAttributeNames()) if (t7.endsWith(h2)) {
          const i7 = v2[a3++], s5 = r7.getAttribute(t7).split(o3), e8 = /([.?@])?(.*)/.exec(i7);
          d3.push({ type: 1, index: l3, name: e8[2], strings: s5, ctor: "." === e8[1] ? I : "?" === e8[1] ? L : "@" === e8[1] ? z : H }), r7.removeAttribute(t7);
        } else t7.startsWith(o3) && (d3.push({ type: 6, index: l3 }), r7.removeAttribute(t7));
        if (y2.test(r7.tagName)) {
          const t7 = r7.textContent.split(o3), i7 = t7.length - 1;
          if (i7 > 0) {
            r7.textContent = s2 ? s2.emptyScript : "";
            for (let s5 = 0; s5 < i7; s5++) r7.append(t7[s5], c3()), P.nextNode(), d3.push({ type: 2, index: ++l3 });
            r7.append(t7[i7], c3());
          }
        }
      } else if (8 === r7.nodeType) if (r7.data === n3) d3.push({ type: 2, index: l3 });
      else {
        let t7 = -1;
        for (; -1 !== (t7 = r7.data.indexOf(o3, t7 + 1)); ) d3.push({ type: 7, index: l3 }), t7 += o3.length - 1;
      }
      l3++;
    }
  }
  static createElement(t6, i6) {
    const s5 = l2.createElement("template");
    return s5.innerHTML = t6, s5;
  }
};
function M(t6, i6, s5 = t6, e7) {
  if (i6 === E) return i6;
  let h5 = void 0 !== e7 ? s5._$Co?.[e7] : s5._$Cl;
  const o8 = a2(i6) ? void 0 : i6._$litDirective$;
  return h5?.constructor !== o8 && (h5?._$AO?.(false), void 0 === o8 ? h5 = void 0 : (h5 = new o8(t6), h5._$AT(t6, s5, e7)), void 0 !== e7 ? (s5._$Co ??= [])[e7] = h5 : s5._$Cl = h5), void 0 !== h5 && (i6 = M(t6, h5._$AS(t6, i6.values), h5, e7)), i6;
}
var R = class {
  constructor(t6, i6) {
    this._$AV = [], this._$AN = void 0, this._$AD = t6, this._$AM = i6;
  }
  get parentNode() {
    return this._$AM.parentNode;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  u(t6) {
    const { el: { content: i6 }, parts: s5 } = this._$AD, e7 = (t6?.creationScope ?? l2).importNode(i6, true);
    P.currentNode = e7;
    let h5 = P.nextNode(), o8 = 0, n7 = 0, r7 = s5[0];
    for (; void 0 !== r7; ) {
      if (o8 === r7.index) {
        let i7;
        2 === r7.type ? i7 = new k(h5, h5.nextSibling, this, t6) : 1 === r7.type ? i7 = new r7.ctor(h5, r7.name, r7.strings, this, t6) : 6 === r7.type && (i7 = new Z(h5, this, t6)), this._$AV.push(i7), r7 = s5[++n7];
      }
      o8 !== r7?.index && (h5 = P.nextNode(), o8++);
    }
    return P.currentNode = l2, e7;
  }
  p(t6) {
    let i6 = 0;
    for (const s5 of this._$AV) void 0 !== s5 && (void 0 !== s5.strings ? (s5._$AI(t6, s5, i6), i6 += s5.strings.length - 2) : s5._$AI(t6[i6])), i6++;
  }
};
var k = class _k {
  get _$AU() {
    return this._$AM?._$AU ?? this._$Cv;
  }
  constructor(t6, i6, s5, e7) {
    this.type = 2, this._$AH = A, this._$AN = void 0, this._$AA = t6, this._$AB = i6, this._$AM = s5, this.options = e7, this._$Cv = e7?.isConnected ?? true;
  }
  get parentNode() {
    let t6 = this._$AA.parentNode;
    const i6 = this._$AM;
    return void 0 !== i6 && 11 === t6?.nodeType && (t6 = i6.parentNode), t6;
  }
  get startNode() {
    return this._$AA;
  }
  get endNode() {
    return this._$AB;
  }
  _$AI(t6, i6 = this) {
    t6 = M(this, t6, i6), a2(t6) ? t6 === A || null == t6 || "" === t6 ? (this._$AH !== A && this._$AR(), this._$AH = A) : t6 !== this._$AH && t6 !== E && this._(t6) : void 0 !== t6._$litType$ ? this.$(t6) : void 0 !== t6.nodeType ? this.T(t6) : d2(t6) ? this.k(t6) : this._(t6);
  }
  O(t6) {
    return this._$AA.parentNode.insertBefore(t6, this._$AB);
  }
  T(t6) {
    this._$AH !== t6 && (this._$AR(), this._$AH = this.O(t6));
  }
  _(t6) {
    this._$AH !== A && a2(this._$AH) ? this._$AA.nextSibling.data = t6 : this.T(l2.createTextNode(t6)), this._$AH = t6;
  }
  $(t6) {
    const { values: i6, _$litType$: s5 } = t6, e7 = "number" == typeof s5 ? this._$AC(t6) : (void 0 === s5.el && (s5.el = S2.createElement(V(s5.h, s5.h[0]), this.options)), s5);
    if (this._$AH?._$AD === e7) this._$AH.p(i6);
    else {
      const t7 = new R(e7, this), s6 = t7.u(this.options);
      t7.p(i6), this.T(s6), this._$AH = t7;
    }
  }
  _$AC(t6) {
    let i6 = C.get(t6.strings);
    return void 0 === i6 && C.set(t6.strings, i6 = new S2(t6)), i6;
  }
  k(t6) {
    u2(this._$AH) || (this._$AH = [], this._$AR());
    const i6 = this._$AH;
    let s5, e7 = 0;
    for (const h5 of t6) e7 === i6.length ? i6.push(s5 = new _k(this.O(c3()), this.O(c3()), this, this.options)) : s5 = i6[e7], s5._$AI(h5), e7++;
    e7 < i6.length && (this._$AR(s5 && s5._$AB.nextSibling, e7), i6.length = e7);
  }
  _$AR(t6 = this._$AA.nextSibling, s5) {
    for (this._$AP?.(false, true, s5); t6 !== this._$AB; ) {
      const s6 = i3(t6).nextSibling;
      i3(t6).remove(), t6 = s6;
    }
  }
  setConnected(t6) {
    void 0 === this._$AM && (this._$Cv = t6, this._$AP?.(t6));
  }
};
var H = class {
  get tagName() {
    return this.element.tagName;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  constructor(t6, i6, s5, e7, h5) {
    this.type = 1, this._$AH = A, this._$AN = void 0, this.element = t6, this.name = i6, this._$AM = e7, this.options = h5, s5.length > 2 || "" !== s5[0] || "" !== s5[1] ? (this._$AH = Array(s5.length - 1).fill(new String()), this.strings = s5) : this._$AH = A;
  }
  _$AI(t6, i6 = this, s5, e7) {
    const h5 = this.strings;
    let o8 = false;
    if (void 0 === h5) t6 = M(this, t6, i6, 0), o8 = !a2(t6) || t6 !== this._$AH && t6 !== E, o8 && (this._$AH = t6);
    else {
      const e8 = t6;
      let n7, r7;
      for (t6 = h5[0], n7 = 0; n7 < h5.length - 1; n7++) r7 = M(this, e8[s5 + n7], i6, n7), r7 === E && (r7 = this._$AH[n7]), o8 ||= !a2(r7) || r7 !== this._$AH[n7], r7 === A ? t6 = A : t6 !== A && (t6 += (r7 ?? "") + h5[n7 + 1]), this._$AH[n7] = r7;
    }
    o8 && !e7 && this.j(t6);
  }
  j(t6) {
    t6 === A ? this.element.removeAttribute(this.name) : this.element.setAttribute(this.name, t6 ?? "");
  }
};
var I = class extends H {
  constructor() {
    super(...arguments), this.type = 3;
  }
  j(t6) {
    this.element[this.name] = t6 === A ? void 0 : t6;
  }
};
var L = class extends H {
  constructor() {
    super(...arguments), this.type = 4;
  }
  j(t6) {
    this.element.toggleAttribute(this.name, !!t6 && t6 !== A);
  }
};
var z = class extends H {
  constructor(t6, i6, s5, e7, h5) {
    super(t6, i6, s5, e7, h5), this.type = 5;
  }
  _$AI(t6, i6 = this) {
    if ((t6 = M(this, t6, i6, 0) ?? A) === E) return;
    const s5 = this._$AH, e7 = t6 === A && s5 !== A || t6.capture !== s5.capture || t6.once !== s5.once || t6.passive !== s5.passive, h5 = t6 !== A && (s5 === A || e7);
    e7 && this.element.removeEventListener(this.name, this, s5), h5 && this.element.addEventListener(this.name, this, t6), this._$AH = t6;
  }
  handleEvent(t6) {
    "function" == typeof this._$AH ? this._$AH.call(this.options?.host ?? this.element, t6) : this._$AH.handleEvent(t6);
  }
};
var Z = class {
  constructor(t6, i6, s5) {
    this.element = t6, this.type = 6, this._$AN = void 0, this._$AM = i6, this.options = s5;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AI(t6) {
    M(this, t6);
  }
};
var j = { M: h2, P: o3, A: n3, C: 1, L: N, R, D: d2, V: M, I: k, H, N: L, U: z, B: I, F: Z };
var B = t2.litHtmlPolyfillSupport;
B?.(S2, k), (t2.litHtmlVersions ??= []).push("3.3.2");
var D = (t6, i6, s5) => {
  const e7 = s5?.renderBefore ?? i6;
  let h5 = e7._$litPart$;
  if (void 0 === h5) {
    const t7 = s5?.renderBefore ?? null;
    e7._$litPart$ = h5 = new k(i6.insertBefore(c3(), t7), t7, void 0, s5 ?? {});
  }
  return h5._$AI(t6), h5;
};

// ../node_modules/lit-element/lit-element.js
var s3 = globalThis;
var i4 = class extends y {
  constructor() {
    super(...arguments), this.renderOptions = { host: this }, this._$Do = void 0;
  }
  createRenderRoot() {
    const t6 = super.createRenderRoot();
    return this.renderOptions.renderBefore ??= t6.firstChild, t6;
  }
  update(t6) {
    const r7 = this.render();
    this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(t6), this._$Do = D(r7, this.renderRoot, this.renderOptions);
  }
  connectedCallback() {
    super.connectedCallback(), this._$Do?.setConnected(true);
  }
  disconnectedCallback() {
    super.disconnectedCallback(), this._$Do?.setConnected(false);
  }
  render() {
    return E;
  }
};
i4._$litElement$ = true, i4["finalized"] = true, s3.litElementHydrateSupport?.({ LitElement: i4 });
var o4 = s3.litElementPolyfillSupport;
o4?.({ LitElement: i4 });
(s3.litElementVersions ??= []).push("4.2.2");

// ../node_modules/@lit/reactive-element/decorators/custom-element.js
var t3 = (t6) => (e7, o8) => {
  void 0 !== o8 ? o8.addInitializer(() => {
    customElements.define(t6, e7);
  }) : customElements.define(t6, e7);
};

// ../node_modules/@lit/reactive-element/decorators/property.js
var o5 = { attribute: true, type: String, converter: u, reflect: false, hasChanged: f };
var r4 = (t6 = o5, e7, r7) => {
  const { kind: n7, metadata: i6 } = r7;
  let s5 = globalThis.litPropertyMetadata.get(i6);
  if (void 0 === s5 && globalThis.litPropertyMetadata.set(i6, s5 = /* @__PURE__ */ new Map()), "setter" === n7 && ((t6 = Object.create(t6)).wrapped = true), s5.set(r7.name, t6), "accessor" === n7) {
    const { name: o8 } = r7;
    return { set(r8) {
      const n8 = e7.get.call(this);
      e7.set.call(this, r8), this.requestUpdate(o8, n8, t6, true, r8);
    }, init(e8) {
      return void 0 !== e8 && this.C(o8, void 0, t6, e8), e8;
    } };
  }
  if ("setter" === n7) {
    const { name: o8 } = r7;
    return function(r8) {
      const n8 = this[o8];
      e7.call(this, r8), this.requestUpdate(o8, n8, t6, true, r8);
    };
  }
  throw Error("Unsupported decorator location: " + n7);
};
function n4(t6) {
  return (e7, o8) => "object" == typeof o8 ? r4(t6, e7, o8) : ((t7, e8, o9) => {
    const r7 = e8.hasOwnProperty(o9);
    return e8.constructor.createProperty(o9, t7), r7 ? Object.getOwnPropertyDescriptor(e8, o9) : void 0;
  })(t6, e7, o8);
}

// src/components/bs-debug.ts
var BsDebug = class extends i4 {
  constructor() {
    super(...arguments);
    this.servers = { servers: [] };
    this.me = { routes: [], id: "" };
  }
  get otherServers() {
    return this.servers.servers.filter(
      (server) => server.id !== this.me.id
    );
  }
  render() {
    return b2`
            <bs-header></bs-header>
            <bs-server-detail .server=${this.me}></bs-server-detail>
            ${this.otherServers.length > 0 ? b2` <bs-server-list
                      .servers=${this.otherServers}
                  ></bs-server-list>` : null}
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
    * {
        box-sizing: border-box;
    }
    pre {
        margin: 0;
        padding: 0;
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
var BsServerList = class extends i4 {
  constructor() {
    super(...arguments);
    this.servers = [];
  }
  static {
    this.styles = [base, i``];
  }
  render() {
    return b2`
            ${this.servers.map((server) => {
      const display_addr = "http://" + server.socket_addr;
      let url = new URL(display_addr);
      let bs_url = new URL("./__bslive", display_addr);
      return b2`
                    <div>
                        <bs-server-identity
                            .identity=${server.identity}
                        ></bs-server-identity>
                        <p>
                            <a href=${url} target="_blank"
                                ><code>${url}</code></a
                            >
                        </p>
                        <p>
                            <bs-icon icon-name="logo"></bs-icon>
                            <small
                                ><a href=${bs_url} target="_blank"
                                    ><code>${bs_url}</code></a
                                ></small
                            >
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
var BsServerDetail = class extends i4 {
  constructor() {
    super(...arguments);
    this.server = { routes: [], id: "" };
  }
  static {
    this.styles = [base];
  }
  render() {
    return b2`
            <pre><code>${JSON.stringify(this.server, null, 2)}</code></pre>
        `;
  }
};
__decorateClass([
  n4({ type: Object })
], BsServerDetail.prototype, "server", 2);
customElements.define("bs-server-detail", BsServerDetail);

// src/components/bs-server-identity.ts
var BsServerIdentity = class extends i4 {
  static {
    this.styles = [base];
  }
  render() {
    switch (this.identity.kind) {
      case "Named":
      case "Both": {
        return b2`<p>
                    <strong>[named] ${this.identity.payload.name}</strong>
                </p>`;
      }
      default:
        return b2`<p><strong>[unnamed]</strong></p>`;
    }
  }
};
__decorateClass([
  n4({ type: Object })
], BsServerIdentity.prototype, "identity", 2);
customElements.define("bs-server-identity", BsServerIdentity);

// src/components/bs-header.ts
var BsHeader = class extends i4 {
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
    return b2`
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
var BsIcon = class extends i4 {
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
        return b2`<svg class="svg-icon" part="svg">
                    <use xlink:href="#svg-logo"></use>
                </svg>`;
      case "wordmark":
        return b2`<svg class="svg-icon" part="svg">
                    <use xlink:href="#svg-wordmark"></use>
                </svg>`;
      default:
        return `unknown`;
    }
  }
  render() {
    return b2`<svg
                xmlns="http://www.w3.org/2000/svg"
                xmlns:xlink="http://www.w3.org/1999/xlink"
                style="display:none"
            >
                <symbol id="svg-check" viewBox="0 0 20 20">
                    <path
                        d="M8.294 16.998c-.435 0-.847-.203-1.11-.553l-3.574-4.72c-.465-.614-.344-1.487.27-1.952.615-.467 1.488-.344 1.953.27l2.35 3.104 5.912-9.492c.407-.652 1.267-.852 1.92-.445.654.406.855 1.266.447 1.92L9.478 16.34c-.242.39-.66.635-1.12.656-.022.002-.042.002-.064.002z"
                    />
                </symbol>
                <symbol
                    id="svg-creative-commons-noncommercial-us"
                    viewBox="0 0 20 20"
                >
                    <path
                        d="M9.988.4c2.69 0 4.966.928 6.825 2.784C18.67 5.04 19.6 7.312 19.6 10s-.913 4.936-2.74 6.744C14.923 18.648 12.63 19.6 9.99 19.6c-2.61 0-4.862-.944-6.753-2.832C1.345 14.88.4 12.624.4 10s.945-4.896 2.835-6.816C5.078 1.328 7.33.4 9.988.4zM2.56 7.42c-.287.81-.43 1.67-.43 2.58 0 2.128.777 3.968 2.33 5.52 1.555 1.552 3.405 2.328 5.552 2.328s4.013-.784 5.6-2.352c.53-.513.967-1.073 1.31-1.68l-3.618-1.61c-.246 1.216-1.33 2.04-2.643 2.136v1.48h-1.1v-1.48c-1.078-.013-2.12-.453-2.915-1.15l1.322-1.333c.637.598 1.274.868 2.143.868.563 0 1.188-.22 1.188-.955 0-.26-.1-.44-.26-.577l-.915-.407-1.14-.508c-.563-.252-1.04-.464-1.52-.677L2.56 7.42zm7.452-5.292c-2.18 0-4.02.768-5.527 2.304-.41.414-.766.846-1.07 1.297l3.67 1.632c.332-1.017 1.3-1.635 2.474-1.704v-1.48h1.1v1.48c.76.037 1.593.245 2.413.88l-1.26 1.297c-.466-.33-1.054-.563-1.642-.563-.476 0-1.15.148-1.15.747 0 .09.03.17.086.242l1.228.547.83.37c.532.236 1.04.46 1.542.685l4.92 2.19c.162-.644.244-1.33.244-2.055 0-2.192-.77-4.048-2.307-5.568-1.522-1.536-3.372-2.304-5.55-2.304z"
                    />
                </symbol>
                <symbol id="svg-back-in-time" viewBox="0 0 20 20">
                    <path
                        d="M11 1.8c-4.445 0-8.06 3.56-8.17 7.995V10H.46l3.593 3.894L7.547 10H4.875v-.205C4.982 6.492 7.683 3.85 11 3.85c3.386 0 6.13 2.754 6.13 6.15 0 3.396-2.744 6.15-6.13 6.15-1.357 0-2.61-.445-3.627-1.193L5.967 16.46C7.355 17.55 9.102 18.2 11 18.2c4.515 0 8.174-3.67 8.174-8.2S15.514 1.8 11 1.8zM10 5v5c0 .13.027.26.077.382s.124.233.216.325l3.2 3.2c.283-.183.55-.39.787-.628L12 11V5h-2z"
                    />
                </symbol>
                <symbol id="svg-time-slot" viewBox="0 0 20 20">
                    <path
                        d="M10 .4C4.698.4.4 4.698.4 10s4.298 9.6 9.6 9.6c5.3 0 9.6-4.298 9.6-9.6S15.3.4 10 .4zm0 17.2c-4.197 0-7.6-3.403-7.6-7.6C2.4 5.8 5.802 2.4 10 2.4V10l6.792-3.396c.513 1.023.808 2.173.808 3.396 0 4.197-3.403 7.6-7.6 7.6z"
                    />
                </symbol>
                <symbol id="svg-merge" viewBox="0 0 20 20">
                    <path
                        d="M17.89 17.707L16.892 20c-3.137-1.366-5.496-3.152-6.892-5.275-1.396 2.123-3.755 3.91-6.892 5.275l-.998-2.293C5.14 16.39 8.55 14.102 8.55 10V7H5.5L10 0l4.5 7h-3.05v3c0 4.102 3.41 6.39 6.44 7.707z"
                    />
                </symbol>
                <symbol id="svg-text" viewBox="0 0 20 20">
                    <path
                        fill-rule="evenodd"
                        clip-rule="evenodd"
                        d="M15.5 11h-11c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h11c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5zm0-4h-11c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h11c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5zm-5 8h-6c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h6c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5zm5-12h-11c-.275 0-.5.225-.5.5v1c0 .276.225.5.5.5h11c.276 0 .5-.224.5-.5v-1c0-.275-.224-.5-.5-.5z"
                    />
                </symbol>
                <symbol id="svg-tv" viewBox="0 0 20 20">
                    <path
                        d="M18 1H2C.9 1 0 1.9 0 3v11c0 1.1.882 2.178 1.96 2.393l4.373.875S2.57 19 5 19h10c2.43 0-1.334-1.732-1.334-1.732l4.373-.875C19.116 16.178 20 15.1 20 14V3c0-1.1-.9-2-2-2zm0 13H2V3h16v11z"
                    />
                </symbol>
                <symbol id="svg-block" viewBox="0 0 20 20">
                    <path
                        d="M10 .4C4.697.4.4 4.698.4 10c0 5.303 4.297 9.6 9.6 9.6 5.3 0 9.6-4.297 9.6-9.6 0-5.302-4.3-9.6-9.6-9.6zM2.4 10c0-4.197 3.4-7.6 7.6-7.6 1.828 0 3.505.647 4.816 1.723L4.122 14.817C3.046 13.505 2.4 11.83 2.4 10zm7.6 7.6c-1.83 0-3.506-.647-4.816-1.723L15.878 5.184C16.953 6.496 17.6 8.17 17.6 10c0 4.197-3.404 7.6-7.6 7.6z"
                    />
                </symbol>
                <symbol id="svg-list" viewBox="0 0 20 20">
                    <path
                        d="M14.4 9H8.6c-.552 0-.6.447-.6 1s.048 1 .6 1h5.8c.552 0 .6-.447.6-1s-.048-1-.6-1zm2 5H8.6c-.552 0-.6.447-.6 1s.048 1 .6 1h7.8c.552 0 .6-.447.6-1s-.048-1-.6-1zM8.6 6h7.8c.552 0 .6-.447.6-1s-.048-1-.6-1H8.6c-.552 0-.6.447-.6 1s.048 1 .6 1zM5.4 9H3.6c-.552 0-.6.447-.6 1s.048 1 .6 1h1.8c.552 0 .6-.447.6-1s-.048-1-.6-1zm0 5H3.6c-.552 0-.6.447-.6 1s.048 1 .6 1h1.8c.552 0 .6-.447.6-1s-.048-1-.6-1zm0-10H3.6c-.552 0-.6.447-.6 1s.048 1 .6 1h1.8c.552 0 .6-.447.6-1s-.048-1-.6-1z"
                    />
                </symbol>
                <symbol id="svg-logo" viewBox="0 0 140 204.1">
                    <path
                        d="M63.5.3L1.7 31.2c-1 .5-1.7 1.5-1.7 2.7v136.3c0 1.1.6 2.2 1.7 2.7l61.8 30.9c2 1 4.3-.5 4.3-2.7V3c0-2.2-2.3-3.7-4.3-2.7zM76.5 203.8l61.8-30.9c1-.5 1.7-1.5 1.7-2.7v-66.3c0-1.1-.6-2.2-1.7-2.7L76.5 70.3c-2-1-4.3.5-4.3 2.7v128.1c0 2.2 2.3 3.7 4.3 2.7z"
                    />
                </symbol>
                <symbol id="svg-wordmark" viewBox="0 0 536.3 106.8">
                    <path
                        d="M33 .2L.9 16.2c-.6.3-.9.8-.9 1.4v70.8c0 .6.3 1.1.9 1.4l32.1 16c1 .5 2.3-.2 2.3-1.4V1.6C35.2.4 34-.4 33 .2zM39.7 105.8l32.1-16c.5-.3.9-.8.9-1.4V54c0-.6-.3-1.1-.9-1.4l-32.1-16c-1-.5-2.3.2-2.3 1.4v66.5c.1 1.1 1.3 1.8 2.3 1.3zM129.7 34.8c10.8 0 16.6 4 16.6 14.1 0 6.6-2.1 9.8-6.4 12.2 4.7 1.8 7.8 5.2 7.8 12.6 0 11.1-6.7 15.4-17.3 15.4H109V34.8h20.7zm-11.8 7.6V58h11.7c5.4 0 7.8-2.7 7.8-8 0-5.2-2.7-7.5-8.1-7.5h-11.4zm0 23v16.1h12c5.5 0 8.7-1.7 8.7-8.3 0-6.2-4.6-7.9-8.9-7.9h-11.8zM156.6 49.5h8.6v4.8s6.7-4.4 13.5-5.6v8.6c-7.2 1.4-13.4 6.3-13.4 6.3v25.6h-8.6V49.5zM365.4 49.5h8.6v4.8s6.7-4.4 13.5-5.6v8.6c-7.2 1.4-13.4 6.3-13.4 6.3v25.6h-8.6V49.5zM218.4 69.1c0 13.2-4 20.9-17.7 20.9-13.6 0-17.7-7.8-17.7-20.9 0-12.9 4.4-20.5 17.7-20.5s17.7 7.6 17.7 20.5zm-8.7 0c0-9.2-2-13.2-9-13.2s-9 4-9 13.2 1.6 13.6 9 13.6 9-4.4 9-13.6zM232.3 49.5l6.3 32.3h1.6l7.5-31.5h8.9l7.5 31.5h1.6l6.2-32.3h8.6l-8.4 39.7h-13.7L252.2 62 246 89.2h-13.7l-8.4-39.7h8.4zM315.4 57.7s-9.4-1.3-14.1-1.3c-4.8 0-6.9 1.1-6.9 4.4 0 2.6 1.7 3.3 9.4 4.7 9.5 1.7 12.9 4 12.9 12 0 9.3-5.9 12.6-15.7 12.6-5.5 0-14.7-1.7-14.7-1.7l.3-7.2s9.5 1.3 13.6 1.3c5.7 0 7.9-1.2 7.9-4.7 0-2.8-1.3-3.6-9.2-4.9-8.7-1.4-13.2-3.3-13.2-11.7 0-9 7-12.3 14.8-12.3 5.8 0 14.9 1.7 14.9 1.7v7.1zM355.6 81.8l.2 6.4s-9 1.8-16 1.8c-11.9 0-16.5-6.3-16.5-20.3 0-14.5 6.3-21.1 17.2-21.1 11.1 0 16.7 5.8 16.7 18.2l-.6 6.2H332c.1 6.3 2.5 9.5 9 9.5 6.2 0 14.6-.7 14.6-.7zm-7-15.5c0-7.9-2.4-10.6-8.2-10.6-5.9 0-8.5 2.9-8.6 10.6h16.8zM420.5 54.3S412 53 406.8 53c-4.9 0-9.4 1.3-9.4 6.7 0 4.1 2 5.3 10.6 6.7 10.2 1.7 14 3.5 14 11.1 0 9.3-5.8 12.3-15.3 12.3-4.8 0-13.6-1.4-13.6-1.4l.3-4.2s8.9 1.3 12.9 1.3c6.8 0 10.8-1.6 10.8-7.8 0-4.8-2.4-5.8-11.3-7.1-9.1-1.4-13.3-3.1-13.3-10.8 0-8.6 7.1-11.2 14-11.2 6 0 14 1.3 14 1.3v4.4zM432 49.5L442.5 85h2.9L456 49.5h4.8l-16.9 57.3h-4.8l5.1-17.7h-5.5L427 49.4h5zM468.9 89.2V49.5h4.7v2.9s6.7-3.7 12.9-3.7c10.9 0 13.3 5.1 13.3 19.6v20.9H495V68.5c0-11.7-1.3-15.6-9.2-15.6-6.2 0-12.2 3.3-12.2 3.3V89h-4.7zM536.2 49.7l-.2 4s-6.3-.8-9.3-.8c-9.5 0-12.3 4.2-12.3 15.6 0 12.5 1.9 17.1 12.3 17.1 3 0 9.4-.7 9.4-.7l.2 4s-7.1 1-10.5 1c-12.9 0-16.3-5.7-16.3-21.3 0-14.5 4.6-19.9 16.4-19.9 3.4 0 10.3 1 10.3 1z"
                    />
                </symbol>
                <symbol id="svg-github" viewBox="0 0 32 32">
                    <path
                        clip-rule="evenodd"
                        d="M16.003 0C7.17 0 .008 7.162.008 15.997c0 7.067 4.582 13.063 10.94 15.18.8.145 1.052-.33 1.052-.753 0-.38.008-1.442 0-2.777-4.45.967-5.37-2.107-5.37-2.107-.728-1.848-1.776-2.34-1.776-2.34-1.452-.992.11-.973.11-.973 1.604.113 2.45 1.65 2.45 1.65 1.427 2.442 3.743 1.736 4.654 1.328.146-1.034.56-1.74 1.017-2.14C9.533 22.663 5.8 21.29 5.8 15.16c0-1.747.622-3.174 1.645-4.292-.165-.404-.715-2.03.157-4.234 0 0 1.343-.43 4.398 1.64 1.276-.354 2.645-.53 4.005-.537 1.36.006 2.727.183 4.005.538 3.055-2.07 4.396-1.64 4.396-1.64.872 2.202.323 3.83.16 4.233 1.022 1.118 1.643 2.545 1.643 4.292 0 6.146-3.74 7.498-7.305 7.893C19.48 23.548 20 24.508 20 26v4.428c0 .428.258.9 1.07.746C27.422 29.054 32 23.062 32 15.997 32 7.162 24.838 0 16.003 0z"
                        fill-rule="evenodd"
                    />
                </symbol>
                <symbol id="svg-twitter" viewBox="0 0 273.4 222.2">
                    <path
                        d="M273.4 26.3c-10.1 4.5-20.9 7.5-32.2 8.8 11.6-6.9 20.5-17.9 24.7-31-10.9 6.4-22.9 11.1-35.7 13.6C220 6.8 205.4 0 189.3 0c-31 0-56.1 25.1-56.1 56.1 0 4.4.5 8.7 1.5 12.8C88 66.5 46.7 44.2 19 10.3c-4.8 8.3-7.6 17.9-7.6 28.2 0 19.5 9.9 36.6 25 46.7-9.2-.3-17.8-2.8-25.4-7v.7c0 27.2 19.3 49.8 45 55-4.7 1.3-9.7 2-14.8 2-3.6 0-7.1-.4-10.6-1 7.1 22.3 27.9 38.5 52.4 39-19.2 15-43.4 24-69.7 24-4.5 0-9-.3-13.4-.8 24.8 15.9 54.3 25.2 86 25.2 103.2 0 159.6-85.5 159.6-159.6 0-2.4-.1-4.9-.2-7.3 11.1-8 20.6-17.9 28.1-29.1z"
                    />
                </symbol>
                <symbol id="svg-circle-play" viewBox="0 0 191.4 191.4">
                    <circle
                        fill="none"
                        stroke="#FFF"
                        stroke-width="22"
                        stroke-miterlimit="10"
                        cx="95.7"
                        cy="95.7"
                        r="84.7"
                    />
                    <path
                        d="M87.8 57l46.7 32.6c4.2 3 4.2 9.2 0 12.2l-45.3 31.6c-4.7 3.3-11.1-.1-11.1-5.8V62c0-4.9 5.6-7.9 9.7-5z"
                    />
                </symbol>
                <symbol id="svg-code" viewBox="0 0 20 20">
                    <path
                        d="M5.72 14.75c-.237 0-.475-.083-.665-.252L-.005 10l5.34-4.748c.413-.365 1.045-.33 1.412.083.367.413.33 1.045-.083 1.412L3.004 10l3.38 3.002c.412.367.45 1 .082 1.412-.197.223-.472.336-.747.336zm8.944-.002L20.004 10l-5.06-4.498c-.412-.367-1.044-.33-1.41.083-.367.413-.33 1.045.083 1.412L16.995 10l-3.66 3.252c-.412.367-.45 1-.082 1.412.197.223.472.336.747.336.236 0 .474-.083.664-.252zm-4.678 1.417l2-12c.09-.545-.277-1.06-.822-1.15-.547-.093-1.06.276-1.15.82l-2 12c-.09.546.277 1.06.822 1.152.056.01.11.013.165.013.48 0 .905-.347.986-.835z"
                    />
                </symbol>
                <symbol id="svg-menu" viewBox="0 0 20 20">
                    <path
                        d="M16.4 9H3.6c-.552 0-.6.447-.6 1 0 .553.048 1 .6 1h12.8c.552 0 .6-.447.6-1 0-.553-.048-1-.6-1zm0 4H3.6c-.552 0-.6.447-.6 1 0 .553.048 1 .6 1h12.8c.552 0 .6-.447.6-1 0-.553-.048-1-.6-1zM3.6 7h12.8c.552 0 .6-.447.6-1 0-.553-.048-1-.6-1H3.6c-.552 0-.6.447-.6 1 0 .553.048 1 .6 1z"
                    />
                </symbol>
                <symbol id="svg-cross" viewBox="0 0 20 20">
                    <path
                        d="M14.348 14.85c-.47.468-1.23.468-1.697 0L10 11.82l-2.65 3.028c-.47.47-1.23.47-1.698 0-.47-.47-.47-1.23 0-1.697L8.41 10 5.65 6.85c-.468-.47-.468-1.23 0-1.698.47-.47 1.23-.47 1.698 0L10 8.182l2.65-3.03c.47-.47 1.23-.47 1.698 0 .47.47.47 1.23 0 1.697L11.59 10l2.758 3.15c.47.47.47 1.23 0 1.7z"
                    />
                </symbol>
                <symbol id="svg-typeface-reg" viewBox="0 0 113.8 77.2">
                    <path
                        d="M20.9 0h18.5l20.9 76.1h-8.4l-5.5-19.6H13.9L8.4 76.1H0L20.9 0zm-5.2 49h28.8L33 7.3h-5.7L15.7 49zM107.5 65.9c.2 3.2 2.9 4.4 6.4 4.8l-.3 6.5c-5.8 0-9.8-1.1-13.1-4.4 0 0-9.9 4.4-19.8 4.4-10 0-15.5-5.7-15.5-16.8 0-10.6 5.5-15.2 16.8-16.3l17.3-1.6v-4.7c0-7.7-3.3-10.5-9.9-10.5-7.7 0-20.8 1.4-20.8 1.4l-.3-6.3S80.4 20 89.9 20c12.4 0 17.7 5.7 17.7 17.7v28.2zM82.9 50.3c-6.7.7-9.4 3.9-9.4 9.9 0 6.4 2.8 10.1 8.4 10.1 8.1 0 17.3-3.4 17.3-3.4V48.7l-16.3 1.6z"
                    />
                </symbol>
                <symbol id="svg-typeface-bold" viewBox="0 0 114.3 76.6">
                    <path
                        d="M18.6 0h24.3l18.7 75.4H49.3l-4.1-16.2H16.3l-4.1 16.2H0L18.6 0zm.1 48.4h24.1l-9.2-38.2H28l-9.3 38.2zM109.5 62.4c.2 3.3 1.7 4.6 4.8 5.1l-.3 9.1c-6.7 0-10.6-.9-14.6-4.1 0 0-8.8 4.1-17.7 4.1-10.9 0-16.4-6-16.4-17.5 0-11.7 6.4-15.6 18.1-16.6l14.2-1.2v-4c0-6.1-2.6-7.9-8-7.9-7.4 0-20.7 1.1-20.7 1.1l-.5-8.5s12-2.9 22.1-2.9c13.4 0 18.9 5.6 18.9 18.2v25.1zM84.8 50.9c-5.1.4-7.6 2.9-7.6 7.8s2.1 8 6.7 8c6.3 0 13.6-2.4 13.6-2.4V49.7l-12.7 1.2z"
                    />
                </symbol>
                <symbol id="svg-typeface-thin" viewBox="0 0 113.3 78">
                    <path
                        d="M23.6 0h11.7L59 77h-4l-7.2-23.6H11.1L4 77H0L23.6 0zM12.3 49.6h34.3l-14-45.9h-6.2L12.3 49.6zM105 69.9c.3 3.2 4.4 4.3 8.2 4.6l-.2 3.4c-4.7 0-8.9-1.2-11.3-4.6 0 0-11.2 4.7-22.2 4.7-9 0-14.4-5.4-14.4-16.1 0-9.3 4.4-14.7 15.2-15.8l20.9-2.2v-5.7c0-9.6-4.2-13.4-12.2-13.4s-20.8 1.9-20.8 1.9l-.3-3.7s12.3-2 21.1-2c11.1 0 16.1 5.8 16.1 17.2v31.7zM80.7 49.5c-8.6.9-11.5 4.8-11.5 12.4 0 8 3.7 12.5 10.5 12.5 10.3 0 21.6-4.5 21.6-4.5V47.4l-20.6 2.1z"
                    />
                </symbol>
            </svg>
            ${this.icon}`;
  }
};
__decorateClass([
  n4({ type: String, attribute: "icon-name" })
], BsIcon.prototype, "iconName", 2);
customElements.define("bs-icon", BsIcon);
function logo() {
  return b2`<bs-icon icon-name="logo"></bs-icon>`;
}

// styles/tokens.ts
var tokens = i`
    :host {
        --brand-blue: #0f2634;
        --brand-grey: #6d6d6d;
        --brand-red: #f24747;
        --brand-white: #ffffff;

        --theme-txt-color: var(--brand-blue);
        --theme-page-color: var(--brand-white);
        --theme-font-family: -apple-system, BlinkMacSystemFont, "Segoe UI",
            Roboto, Oxygen, Ubuntu, Cantarell, "Open Sans", "Helvetica Neue",
            sans-serif;
    }
`;

// src/components/bs-panel.ts
var Panel = class extends i4 {
  constructor() {
    super(...arguments);
    this.title = "...";
  }
  render() {
    return b2`<div class="root">
            <header class="header">
                ${logo()}
                <span class="title">
                    ${this.title}
                <span>
            </header>
            <slot name="detail"></slot></code>
        </div>`;
  }
};
Panel.styles = [
  tokens,
  base,
  i`
            .root {
                background: white;
                color: var(--brand-blue);
                display: grid;
                grid-template-columns: 100%;
                grid-row-gap: 0.6rem;
            }
            slot:has(*) {
                display: block;
                outline: 5px solid red;
            }
            .header {
                display: flex;
                align-items: center;
                gap: 0.4rem;
            }
            ::slotted(*) {
                max-width: 100%;
                overflow-x: auto;
            }
            bs-icon {
                --bs-icon-height: 24px;
                --bs-icon-width: 24px;
                --bs-icon-color: var(--brand-red);
                display: inline-block;
                color: var(--brand-red);
                position: relative;
                top: -2px;
            }
            .title {
                font-size: 0.8rem;
                font-family: var(--theme-font-family);
            }
            slot::slotted(*) {
                font-size: 10px;
            }
        `
];
__decorateClass([
  n4({ type: String })
], Panel.prototype, "title", 2);
Panel = __decorateClass([
  t3("bs-panel")
], Panel);

// ../node_modules/lit-html/directive-helpers.js
var { I: t4 } = j;
var r5 = (o8) => void 0 === o8.strings;

// ../node_modules/lit-html/directive.js
var t5 = { ATTRIBUTE: 1, CHILD: 2, PROPERTY: 3, BOOLEAN_ATTRIBUTE: 4, EVENT: 5, ELEMENT: 6 };
var e5 = (t6) => (...e7) => ({ _$litDirective$: t6, values: e7 });
var i5 = class {
  constructor(t6) {
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AT(t6, e7, i6) {
    this._$Ct = t6, this._$AM = e7, this._$Ci = i6;
  }
  _$AS(t6, e7) {
    return this.update(t6, e7);
  }
  update(t6, e7) {
    return this.render(...e7);
  }
};

// ../node_modules/lit-html/async-directive.js
var s4 = (i6, t6) => {
  const e7 = i6._$AN;
  if (void 0 === e7) return false;
  for (const i7 of e7) i7._$AO?.(t6, false), s4(i7, t6);
  return true;
};
var o6 = (i6) => {
  let t6, e7;
  do {
    if (void 0 === (t6 = i6._$AM)) break;
    e7 = t6._$AN, e7.delete(i6), i6 = t6;
  } while (0 === e7?.size);
};
var r6 = (i6) => {
  for (let t6; t6 = i6._$AM; i6 = t6) {
    let e7 = t6._$AN;
    if (void 0 === e7) t6._$AN = e7 = /* @__PURE__ */ new Set();
    else if (e7.has(i6)) break;
    e7.add(i6), c4(t6);
  }
};
function h3(i6) {
  void 0 !== this._$AN ? (o6(this), this._$AM = i6, r6(this)) : this._$AM = i6;
}
function n5(i6, t6 = false, e7 = 0) {
  const r7 = this._$AH, h5 = this._$AN;
  if (void 0 !== h5 && 0 !== h5.size) if (t6) if (Array.isArray(r7)) for (let i7 = e7; i7 < r7.length; i7++) s4(r7[i7], false), o6(r7[i7]);
  else null != r7 && (s4(r7, false), o6(r7));
  else s4(this, i6);
}
var c4 = (i6) => {
  i6.type == t5.CHILD && (i6._$AP ??= n5, i6._$AQ ??= h3);
};
var f3 = class extends i5 {
  constructor() {
    super(...arguments), this._$AN = void 0;
  }
  _$AT(i6, t6, e7) {
    super._$AT(i6, t6, e7), r6(this), this.isConnected = i6._$AU;
  }
  _$AO(i6, t6 = true) {
    i6 !== this.isConnected && (this.isConnected = i6, i6 ? this.reconnected?.() : this.disconnected?.()), t6 && (s4(this, i6), o6(this));
  }
  setValue(t6) {
    if (r5(this._$Ct)) this._$Ct._$AI(t6, this);
    else {
      const i6 = [...this._$Ct._$AH];
      i6[this._$Ci] = t6, this._$Ct._$AI(i6, this, 0);
    }
  }
  disconnected() {
  }
  reconnected() {
  }
};

// ../node_modules/lit-html/directives/ref.js
var e6 = () => new h4();
var h4 = class {
};
var o7 = /* @__PURE__ */ new WeakMap();
var n6 = e5(class extends f3 {
  render(i6) {
    return A;
  }
  update(i6, [s5]) {
    const e7 = s5 !== this.G;
    return e7 && void 0 !== this.G && this.rt(void 0), (e7 || this.lt !== this.ct) && (this.G = s5, this.ht = i6.options?.host, this.rt(this.ct = i6.element)), A;
  }
  rt(t6) {
    if (this.isConnected || (t6 = void 0), "function" == typeof this.G) {
      const i6 = this.ht ?? globalThis;
      let s5 = o7.get(i6);
      void 0 === s5 && (s5 = /* @__PURE__ */ new WeakMap(), o7.set(i6, s5)), void 0 !== s5.get(this.G) && this.G.call(this.ht, void 0), s5.set(this.G, t6), void 0 !== t6 && this.G.call(this.ht, t6);
    } else this.G.value = t6;
  }
  get lt() {
    return "function" == typeof this.G ? o7.get(this.ht ?? globalThis)?.get(this.G) : this.G?.value;
  }
  disconnected() {
    this.lt === this.ct && this.rt(void 0);
  }
  reconnected() {
    this.rt(this.ct);
  }
});

// src/components/bs-overlay.ts
var WIDTH_NARROW = "narrow";
var WIDTH_WIDE = "wide";
var Overlay = class extends i4 {
  constructor() {
    super(...arguments);
    this.kind = "overlay";
    this.dialogRef = e6();
    this.width = "narrow";
  }
  firstUpdated(_changedProperties) {
    super.firstUpdated(_changedProperties);
    this.dialogRef.value?.showModal();
  }
  closed() {
    this.dispatchEvent(
      new Event("closed", { bubbles: true, composed: true })
    );
  }
  toggleWidth(evt) {
    if (evt.currentTarget instanceof HTMLButtonElement) {
      const val = evt.currentTarget.value;
      if (val === WIDTH_WIDE || val === WIDTH_NARROW) {
        this.width = val;
      }
    }
  }
  render() {
    return b2`
            <dialog
                id="my-dialog"
                ${n6(this.dialogRef)}
                @close=${this.closed}
                data-variant=${this.width}
            >
                <slot name="content"></slot>
                <div class="footer" part="footer">
                    <button
                        @click=${this.toggleWidth}
                        value=${WIDTH_NARROW}
                        part="width-toggle"
                    >
                        ↔️ Narrow
                    </button>
                    <button
                        @click=${this.toggleWidth}
                        value=${WIDTH_WIDE}
                        part="width-toggle"
                    >
                        ↕️ Wide
                    </button>
                    <button
                        commandfor="my-dialog"
                        command="close"
                        part="button"
                    >
                        ✕ Close
                    </button>
                </div>
            </dialog>
        `;
  }
};
Overlay.styles = [
  tokens,
  base,
  i`
            ::backdrop {
                background: rgba(0, 0, 0, 0.7);
            }

            dialog {
                max-height: 90vh;
                max-width: 90vw;
            }

            dialog[data-variant="narrow"] {
                width: 550px;

                button[value="narrow"] {
                    display: none;
                }
            }

            dialog[data-variant="wide"] {
                width: 90vw;

                button[value="wide"] {
                    display: none;
                }
            }

            button {
            }

            .footer {
                display: flex;
                margin-top: 12px;
                justify-content: flex-end;
                gap: 0.5rem;
            }
        `
];
__decorateClass([
  n4({ type: String })
], Overlay.prototype, "kind", 2);
__decorateClass([
  n4({ type: String })
], Overlay.prototype, "width", 2);
Overlay = __decorateClass([
  t3("bs-overlay")
], Overlay);

// src/fixtures/text.ts
var LARGE_CODE = `Lorem ipsum dolor sit amet, consectetur adipisicing elit. Enim laudantium obcaecati voluptatem? Accusantium ad beatae dolorum, fugiat neque rem saepe totam ut. Commodi ex ipsam laudantium obcaecati reiciendis velit, vero!
Lorem ipsum dolor sit amet, consectetur adipisicing elit. Enim laudantium obcaecati voluptatem? Accusantium ad beatae dolorum, fugiat neque rem saepe totam ut. Commodi ex ipsam laudantium obcaecati reiciendis velit, vero!
Lorem ipsum dolor sit amet, consectetur adipisicing elit. Enim laudantium obcaecati voluptatem? Accusantium ad beatae dolorum, fugiat neque rem saepe totam ut. Commodi ex ipsam laudantium obcaecati reiciendis velit, vero!
Lorem ipsum dolor sit amet, consectetur adipisicing elit. Enim laudantium obcaecati voluptatem? Accusantium ad beatae dolorum, fugiat neque rem saepe totam ut. Commodi ex ipsam laudantium obcaecati reiciendis velit, vero!
Lorem ipsum dolor sit amet, consectetur adipisicing elit. Enim laudantium obcaecati voluptatem? Accusantium ad beatae dolorum, fugiat neque rem saepe totam ut. Commodi ex ipsam laudantium obcaecati reiciendis velit, vero!
Lorem ipsum dolor sit amet, consectetur adipisicing elit. Enim laudantium obcaecati voluptatem? Accusantium ad beatae dolorum, fugiat neque rem saepe totam ut. Commodi ex ipsam laudantium obcaecati reiciendis velit, vero!`;

// src/pages/dev.ts
var DevPage = class extends i4 {
  constructor() {
    super(...arguments);
    this.modalShowing = new URLSearchParams(location.search).has("showModal");
    this.timedModalShowing = new URLSearchParams(location.search).has(
      "showTimedModal"
    );
    this._timer = void 0;
  }
  showTimedModal() {
    this.timedModalShowing = true;
    clearTimeout(this._timer);
    this._timer = setTimeout(() => {
      this.timedModalShowing = false;
      this.requestUpdate();
    }, 2e3);
  }
  showModal() {
    this.modalShowing = true;
  }
  closed() {
    this.modalShowing = false;
  }
  get autoOpenLink() {
    const url = new URL(location.href);
    url.searchParams.set("showModal", "true");
    return url.toString();
  }
  render() {
    return b2`
            <h2>Overlays</h2>
            <div class="stack">
                <div>
                    <button @click=${this.showModal}>Show modal</button>
                    <a href="${this.autoOpenLink}">Auto open</a>
                </div>
                <div>
                    <button @click=${this.showTimedModal}>
                        Show timed modal
                    </button>
                </div>
                <bs-panel title="Single line">
                    <pre
                        slot="detail"
                    ><code>A problem has occured in file <b>index.html</b></code></pre>
                </bs-panel>
                <bs-panel title="With detail">
                    <pre slot="detail"><code>${LARGE_CODE}</code></pre>
                </bs-panel>
                <bs-panel title="without any detail"></bs-panel>
            </div>
            <h2>Icons</h2>
            <p>
                <bs-icon icon-name="logo"></bs-icon>
            </p>
            <p>
                <bs-icon icon-name="wordmark"></bs-icon>
            </p>

            ${this.modalShowing ? b2`<bs-overlay title="Single line" @closed=${this.closed}>
                      <bs-panel title="Single line" slot="content">
                          <pre slot="detail">${LARGE_CODE}</pre>
                      </bs-panel>
                  </bs-overlay>` : null}
            ${this.timedModalShowing ? b2`<bs-overlay @closed=${this.closed}>
                      <bs-panel title="Timed Modal" slot="content">
                          <pre
                              slot="detail"
                          ><code>A problem has occured in file <b>index.html</b></code></pre>
                      </bs-panel>
                  </bs-overlay>` : null}
        `;
  }
};
DevPage.styles = [
  base,
  i`
            .stack {
                display: grid;
                width: 100%;
                grid-row-gap: 0.5rem;
            }
            bs-icon[icon-name="wordmark"]::part(svg) {
                height: 30px;
                width: 140px;
            }
            ::slotted {
                border: 5px solid green;
            }
            bs-overlay {
                max-width: 200px;
            }
        `
];
__decorateClass([
  n4({ type: Boolean })
], DevPage.prototype, "modalShowing", 2);
__decorateClass([
  n4({ type: Boolean })
], DevPage.prototype, "timedModalShowing", 2);
DevPage = __decorateClass([
  t3("bs-dev-page")
], DevPage);

// src/index.ts
if (location.pathname === "/dev.html") {
  devEntry();
} else {
  uientry();
}
function devEntry() {
  let next = b2`<bs-dev-page></bs-dev-page>`;
  let app = document.querySelector("#app");
  if (!app) throw new Error("cannot...");
  D(next, app);
}
function uientry() {
  const all = fetch("/__bs_api/servers").then((x2) => x2.json());
  const me = fetch("/__bs_api/me").then((x2) => x2.json());
  Promise.all([all, me]).then(([servers, me2]) => {
    let next = b2`<bs-debug
                .servers=${servers}
                .me=${me2}
            ></bs-debug>`;
    let app = document.querySelector("#app");
    if (!app) throw new Error("cannot...");
    D(next, app);
  }).catch(console.error);
}
/*! Bundled license information:

@lit/reactive-element/css-tag.js:
  (**
   * @license
   * Copyright 2019 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/reactive-element.js:
lit-html/lit-html.js:
lit-element/lit-element.js:
@lit/reactive-element/decorators/custom-element.js:
@lit/reactive-element/decorators/property.js:
@lit/reactive-element/decorators/state.js:
@lit/reactive-element/decorators/event-options.js:
@lit/reactive-element/decorators/base.js:
@lit/reactive-element/decorators/query.js:
@lit/reactive-element/decorators/query-all.js:
@lit/reactive-element/decorators/query-async.js:
@lit/reactive-element/decorators/query-assigned-nodes.js:
lit-html/directive.js:
lit-html/async-directive.js:
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

@lit/reactive-element/decorators/query-assigned-elements.js:
  (**
   * @license
   * Copyright 2021 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

lit-html/directive-helpers.js:
lit-html/directives/ref.js:
  (**
   * @license
   * Copyright 2020 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)
*/
