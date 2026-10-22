// Whole-program problems: tests whose input is standard input. No browser or Monaco.
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { test } = require('node:test');
const ts = require('typescript');
const vm = require('node:vm');
const path = require('node:path');
const root = path.resolve(__dirname, '..');

const jsx = (type, props) => ({ type, props });

function load(file, modules) {
  const context = {
    exports: {},
    require(name) {
      if (name === 'react/jsx-runtime') return { jsx, jsxs: jsx, Fragment: 'Fragment' };
      if (name in modules) return modules[name];
      throw new Error(`unexpected import ${name}`);
    },
  };
  const code = ts.transpileModule(readFileSync(path.join(root, file), 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020, jsx: ts.JsxEmit.ReactJSX, esModuleInterop: true },
  }).outputText;
  vm.runInNewContext(code, context);
  return context.exports;
}

const tests = load('components/problem/submission/tests/index.tsx', {
  'next/dynamic': () => () => null,
  './graph': {}, './grid': {}, './list': {}, './single': {},
});

const program = stdin => ({ name: 'main', arguments: [{ String: { Single: stdin } }], return_type: { String: 'Single' } });

function text(node) {
  if (node === null || node === undefined || node === false) return '';
  if (typeof node === 'string' || typeof node === 'number') return String(node);
  if (Array.isArray(node)) return node.map(text).join('');
  return text(node.props?.children);
}

function find(node, type, found = []) {
  if (!node || typeof node !== 'object') return found;
  if (Array.isArray(node)) { node.forEach(child => find(child, type, found)); return found; }
  if (node.type === type) found.push(node);
  find(node.props?.children, type, found);
  return found;
}

test('program input is the text of a single main string argument', () => {
  assert.equal(tests.programInput(program('3 14\n')), '3 14\n');
  assert.equal(tests.programInput(program('')), '');
  assert.equal(tests.programInput({ name: 'add', arguments: [{ String: { Single: 'x' } }], return_type: { String: 'Single' } }), undefined);
  assert.equal(tests.programInput({ name: 'main', arguments: [{ Int: { Single: 1 } }], return_type: { Int: 'Single' } }), undefined);
  assert.equal(tests.programInput({ name: 'main', arguments: [{ String: { List: ['a'] } }], return_type: { String: 'Single' } }), undefined);
});

test('program examples show standard input and output as plain text', () => {
  const data = {
    parameter_names: [],
    examples: [{ index: 0, input: program('3 14\n'), expected_output: { String: { Single: '5\n' } }, explanation: '' }],
  };
  const examples = load('components/problem/examples.tsx', {
    swr: () => ({ data }),
    '../../utils/fetcher': { api_url: value => value, fetcher: null },
    './submission/tests': tests,
  });
  const blocks = find(examples.default({ id: 1 }), 'pre').map(text);
  assert.deepEqual(blocks, ['3 14\n', '5\n']);
});

test('function examples keep their code-style arguments', () => {
  const data = {
    parameter_names: ['n'],
    examples: [{ index: 0, input: { name: 'f', arguments: [{ Int: { Single: 5 } }], return_type: { Int: 'Single' } }, expected_output: { Int: { Single: 7 } }, explanation: '' }],
  };
  const examples = load('components/problem/examples.tsx', {
    swr: () => ({ data }),
    '../../utils/fetcher': { api_url: value => value, fetcher: null },
    './submission/tests': tests,
  });
  const rendered = examples.default({ id: 1 });
  assert.equal(find(rendered, 'pre').length, 0);
  assert.match(text(rendered), /n = 5/);
  assert.match(text(rendered), /7/);
});
