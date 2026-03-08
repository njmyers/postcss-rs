#!/usr/bin/env node
/**
 * Converts rustdoc JSON output to VitePress-compatible markdown files.
 * Usage: node rustdoc-to-md.mjs <input.json> <output-dir> <crate-name>
 */

import { readFileSync, writeFileSync, mkdirSync, rmSync } from 'fs'
import { join, dirname } from 'path'

const [, , inputFile, outputDir, crateName] = process.argv
if (!inputFile || !outputDir || !crateName) {
  console.error('Usage: rustdoc-to-md.mjs <input.json> <output-dir> <crate-name>')
  process.exit(1)
}

const data = JSON.parse(readFileSync(inputFile, 'utf8'))
const index = data.index

// ─── Type rendering ──────────────────────────────────────────────────────────

function renderType(ty) {
  if (!ty) return '()'
  if (typeof ty === 'string') return ty

  if (ty.resolved_path) {
    const { path, args } = ty.resolved_path
    if (!args) return path
    const angleArgs = args.angle_bracketed
    if (angleArgs) {
      const rendered = angleArgs.args.map(a => {
        if (a.type) return renderType(a.type)
        if (a.lifetime) return `'${a.lifetime}`
        if (a.const) return renderConst(a.const)
        return '?'
      }).join(', ')
      return rendered ? `${path}<${rendered}>` : path
    }
    return path
  }

  if (ty.borrowed_ref) {
    const { lifetime, is_mutable, type: inner } = ty.borrowed_ref
    const lt = lifetime ? `'${lifetime} ` : ''
    const mut = is_mutable ? 'mut ' : ''
    return `&${lt}${mut}${renderType(inner)}`
  }

  if (ty.slice) return `[${renderType(ty.slice)}]`

  if (ty.array) {
    return `[${renderType(ty.array.type)}; ${ty.array.len}]`
  }

  if (ty.tuple) {
    if (ty.tuple.length === 0) return '()'
    return `(${ty.tuple.map(renderType).join(', ')})`
  }

  if (ty.primitive) return ty.primitive

  if (ty.generic) return ty.generic

  if (ty.raw_pointer) {
    const { is_mutable, type: inner } = ty.raw_pointer
    return `*${is_mutable ? 'mut' : 'const'} ${renderType(inner)}`
  }

  if (ty.dyn_trait) {
    const traits = ty.dyn_trait.traits.map(t => t.trait.path).join(' + ')
    const lt = ty.dyn_trait.lifetime ? ` + '${ty.dyn_trait.lifetime}` : ''
    return `dyn ${traits}${lt}`
  }

  if (ty.impl_trait) {
    return `impl ${ty.impl_trait.map(b => b.trait_bound?.trait.path ?? '?').join(' + ')}`
  }

  if (ty.infer !== undefined) return '_'

  return JSON.stringify(ty) // fallback
}

function renderConst(c) {
  if (c.value != null) return c.value
  if (c.expr) return c.expr
  return '_'
}

// ─── Signature rendering ─────────────────────────────────────────────────────

function renderGenerics(generics) {
  if (!generics || !generics.params || generics.params.length === 0) return ''
  const params = generics.params.map(p => {
    if (p.kind.type) {
      const bounds = p.kind.type.bounds
      if (bounds && bounds.length > 0) {
        const bs = bounds.map(b => b.trait_bound?.trait.path ?? '?').join(' + ')
        return `${p.name}: ${bs}`
      }
      return p.name
    }
    if (p.kind.lifetime) return `'${p.name}`
    if (p.kind.const) return `const ${p.name}: ${renderType(p.kind.const.type)}`
    return p.name
  })
  return `<${params.join(', ')}>`
}

function renderFnSig(item) {
  const fn = item.inner.function
  const header = fn.header
  const sig = fn.sig
  const generics = renderGenerics(fn.generics)

  const prefix = [
    header.is_const ? 'const ' : '',
    header.is_async ? 'async ' : '',
    header.is_unsafe ? 'unsafe ' : '',
    'fn ',
  ].join('')

  const inputs = sig.inputs.map(([name, ty]) => `${name}: ${renderType(ty)}`).join(', ')
  const output = sig.output ? ` -> ${renderType(sig.output)}` : ''

  return `${prefix}${item.name}${generics}(${inputs})${output}`
}

function renderStructFields(item) {
  const s = item.inner.struct
  let fields = []
  if (s.kind.plain) {
    fields = s.kind.plain.fields
      .map(id => index[id])
      .filter(Boolean)
      .filter(f => f.visibility === 'public')
  }
  return fields
}

function renderEnumVariants(item) {
  return item.inner.enum.variants
    .map(id => index[id])
    .filter(Boolean)
}

// ─── Markdown helpers ─────────────────────────────────────────────────────────

function badge(text, color) {
  // VitePress custom badge via markdown (use inline HTML minimally)
  return `\`${text}\``
}

function docs(item) {
  if (!item.docs) return ''
  return item.docs.trim() + '\n\n'
}

function anchor(name) {
  return name.toLowerCase().replace(/[^a-z0-9_]/g, '-')
}

// ─── Item renderers ──────────────────────────────────────────────────────────

function renderStruct(item, level = 2) {
  const hashes = '#'.repeat(level)
  const fields = renderStructFields(item)
  let md = `${hashes} \`struct ${item.name}\` {#struct-${anchor(item.name)}}\n\n`
  md += docs(item)

  if (fields.length > 0) {
    md += `**Fields**\n\n`
    for (const f of fields) {
      const ty = renderType(f.inner.struct_field)
      md += `- \`${f.name}: ${ty}\`` + (f.docs ? ` — ${f.docs.trim()}` : '') + '\n'
    }
    md += '\n'
  }
  return md
}

function renderEnum(item, level = 2) {
  const hashes = '#'.repeat(level)
  const variants = renderEnumVariants(item)
  let md = `${hashes} \`enum ${item.name}\` {#enum-${anchor(item.name)}}\n\n`
  md += docs(item)

  if (variants.length > 0) {
    md += `**Variants**\n\n`
    for (const v of variants) {
      const kind = v.inner.variant.kind
      let sig = v.name
      if (kind.tuple) {
        const tys = kind.tuple
          .map(id => index[id])
          .filter(Boolean)
          .map(f => renderType(f.inner.struct_field))
        sig += `(${tys.join(', ')})`
      } else if (kind.struct) {
        const flds = kind.struct.fields
          .map(id => index[id])
          .filter(Boolean)
        if (flds.length > 0) {
          const inner = flds.map(f => `${f.name}: ${renderType(f.inner.struct_field)}`).join(', ')
          sig += ` { ${inner} }`
        }
      }
      md += `- \`${sig}\`` + (v.docs ? ` — ${v.docs.trim()}` : '') + '\n'
    }
    md += '\n'
  }
  return md
}

function renderFunction(item, level = 2) {
  const hashes = '#'.repeat(level)
  let md = `${hashes} \`${renderFnSig(item)}\` {#fn-${anchor(item.name)}}\n\n`
  md += docs(item)
  return md
}

function renderConstant(item, level = 2) {
  const hashes = '#'.repeat(level)
  const c = item.inner.constant
  const ty = renderType(c?.type)
  const val = c?.const?.value ?? c?.const?.expr ?? ''
  let md = `${hashes} \`const ${item.name}: ${ty}\` {#const-${anchor(item.name)}}\n\n`
  md += docs(item)
  if (val) md += `**Value:** \`${val}\`\n\n`
  return md
}

function renderTypeAlias(item, level = 2) {
  const hashes = '#'.repeat(level)
  const ty = renderType(item.inner.type_alias?.type)
  let md = `${hashes} \`type ${item.name} = ${ty}\` {#type-${anchor(item.name)}}\n\n`
  md += docs(item)
  return md
}

function renderItem(item, level = 2) {
  if (!item || item.crate_id !== 0) return ''
  const kind = Object.keys(item.inner)[0]
  if (kind === 'struct') return renderStruct(item, level)
  if (kind === 'enum') return renderEnum(item, level)
  if (kind === 'function') return renderFunction(item, level)
  if (kind === 'constant') return renderConstant(item, level)
  if (kind === 'type_alias') return renderTypeAlias(item, level)
  return ''
}

// ─── Module page generator ───────────────────────────────────────────────────

const PAGE_KINDS = new Set(['struct', 'enum', 'function', 'constant', 'type_alias'])

/**
 * Returns all child modules of a module item, including those that are
 * private but re-exported via `pub use submod::*`.
 */
function getChildModules(moduleItem) {
  const mod = moduleItem.inner.module
  const direct = mod.items
    .map(id => index[id])
    .filter(Boolean)
    .filter(i => i.crate_id === 0 && i.inner.module)

  // Also follow glob re-exports: `pub use submod::*`
  const viaUse = mod.items
    .map(id => index[id])
    .filter(Boolean)
    .filter(i => i.inner.use?.is_glob && i.inner.use?.id != null)
    .map(i => index[i.inner.use.id])
    .filter(Boolean)
    .filter(i => i.crate_id === 0 && i.inner.module)

  // Deduplicate by id
  const seen = new Set(direct.map(i => i.id))
  for (const m of viaUse) {
    if (!seen.has(m.id)) { seen.add(m.id); direct.push(m) }
  }
  return direct
}

/**
 * Returns all public content items (structs, enums, fns, etc.) in a module,
 * including items re-exported via glob uses.
 */
function getModuleItems(moduleItem) {
  const mod = moduleItem.inner.module
  const direct = mod.items
    .map(id => index[id])
    .filter(Boolean)
    .filter(i => i.crate_id === 0 && i.visibility === 'public' && PAGE_KINDS.has(Object.keys(i.inner)[0]))

  // Follow glob re-exports to pull in items from private submodules
  const viaUse = mod.items
    .map(id => index[id])
    .filter(Boolean)
    .filter(i => i.inner.use?.is_glob && i.inner.use?.id != null)
    .flatMap(use_item => {
      const target = index[use_item.inner.use.id]
      if (!target?.inner.module) return []
      return target.inner.module.items
        .map(id => index[id])
        .filter(Boolean)
        .filter(i => i.crate_id === 0 && i.visibility === 'public' && PAGE_KINDS.has(Object.keys(i.inner)[0]))
    })

  const seen = new Set(direct.map(i => i.id))
  for (const item of viaUse) {
    if (!seen.has(item.id)) { seen.add(item.id); direct.push(item) }
  }
  return direct
}

function generateModulePage(moduleItem, modulePath) {
  const title = modulePath.length === 0
    ? crateName
    : modulePath[modulePath.length - 1]

  let md = `---\ntitle: ${title}\n---\n\n`
  md += `# ${title}\n\n`
  md += docs(moduleItem)

  const submodules = getChildModules(moduleItem)
  const items = getModuleItems(moduleItem)

  if (submodules.length > 0) {
    md += `## Modules\n\n`
    for (const m of submodules) {
      md += `- [\`${m.name}\`](./${m.name}) — ${m.docs ? m.docs.split('\n')[0].trim() : ''}\n`
    }
    md += '\n'
  }

  for (const item of items) {
    md += renderItem(item, 2)
  }

  return md
}

// ─── Sidebar entry generator ─────────────────────────────────────────────────

function buildSidebar(moduleItem, basePath, modulePath = []) {
  const submodules = getChildModules(moduleItem)

  const link = modulePath.length === 0
    ? `${basePath}/`
    : `${basePath}/${modulePath.join('/')}`

  const entry = {
    text: modulePath.length === 0 ? crateName : modulePath[modulePath.length - 1],
    link,
    items: submodules.map(m =>
      buildSidebar(m, basePath, [...modulePath, m.name])
    )
  }
  if (entry.items.length === 0) delete entry.items
  return entry
}

// ─── File writer ─────────────────────────────────────────────────────────────

function writeModule(moduleItem, dir, modulePath = []) {
  const pageContent = generateModulePage(moduleItem, modulePath)

  const filePath = modulePath.length === 0
    ? join(dir, 'index.md')
    : join(dir, ...modulePath.slice(0, -1), `${modulePath[modulePath.length - 1]}.md`)

  mkdirSync(dirname(filePath), { recursive: true })
  writeFileSync(filePath, pageContent)

  for (const child of getChildModules(moduleItem)) {
    writeModule(child, dir, [...modulePath, child.name])
  }
}

// ─── Main ────────────────────────────────────────────────────────────────────

const crateDir = join(outputDir, crateName)
rmSync(crateDir, { recursive: true, force: true })
mkdirSync(crateDir, { recursive: true })

const root = index[data.root]
writeModule(root, crateDir)

const sidebar = buildSidebar(root, `/rust-api/${crateName}`)
const sidebarOut = join(outputDir, `${crateName}-sidebar.json`)
writeFileSync(sidebarOut, JSON.stringify(sidebar, null, 2))

console.log(`Written to ${crateDir}`)
console.log(`Sidebar: ${sidebarOut}`)
