import type { languages } from './monaco'

export const DUCKDB_LANGUAGE_ID = 'duckdb'

export const DUCKDB_KEYWORDS = [
  'ALL', 'ALTER', 'AND', 'ANTI', 'ANY', 'AS', 'ASC', 'ASOF', 'ATTACH',
  'BETWEEN', 'BY', 'CASE', 'CAST', 'COLUMNS', 'COMMENT', 'COPY', 'CREATE',
  'CROSS', 'DATABASE', 'DELETE', 'DESC', 'DESCRIBE', 'DETACH', 'DISTINCT',
  'DROP', 'ELSE', 'END', 'ESCAPE', 'EXCEPT', 'EXCLUDE', 'EXISTS', 'EXPLAIN',
  'FALSE', 'FILTER', 'FIRST', 'FOLLOWING', 'FROM', 'FULL', 'GLOB', 'GROUP',
  'GROUPING', 'GROUPS', 'HAVING', 'IF', 'IGNORE', 'ILIKE', 'IN', 'INNER',
  'INSERT', 'INSTALL', 'INTERSECT', 'INTO', 'IS', 'JOIN', 'LAST', 'LATERAL',
  'LEFT', 'LIKE', 'LIMIT', 'LOAD', 'MACRO', 'MAP', 'MATERIALIZED', 'NATURAL',
  'NOT', 'NULL', 'NULLS', 'OFFSET', 'ON', 'OR', 'ORDER', 'OUTER', 'OVER',
  'PARTITION', 'PIVOT', 'POSITIONAL', 'PRECEDING', 'PRAGMA', 'QUALIFY',
  'RANGE', 'RECURSIVE', 'REPLACE', 'RESPECT', 'RETURNING', 'RIGHT', 'ROLLBACK',
  'ROW', 'ROWS', 'SAMPLE', 'SECRET', 'SELECT', 'SEMI', 'SET', 'SHOW',
  'SIMILAR', 'STRUCT', 'SUMMARIZE', 'TABLE', 'TABLESAMPLE', 'TEMP',
  'TEMPORARY', 'THEN', 'TO', 'TRUE', 'UNBOUNDED', 'UNION', 'UNNEST',
  'UNPIVOT', 'UPDATE', 'USE', 'USING', 'VALUES', 'VIEW', 'WHEN', 'WHERE',
  'WINDOW', 'WITH',
]

export const DUCKDB_TYPES = [
  'ARRAY', 'BIGINT', 'BIT', 'BLOB', 'BOOLEAN', 'DATE', 'DECIMAL', 'DOUBLE',
  'FLOAT', 'HUGEINT', 'INTEGER', 'INTERVAL', 'JSON', 'LIST', 'MAP', 'SMALLINT',
  'STRUCT', 'TEXT', 'TIME', 'TIMESTAMP', 'TIMESTAMPTZ', 'TINYINT', 'UBIGINT',
  'UHUGEINT', 'UINTEGER', 'UNION', 'USMALLINT', 'UTINYINT', 'UUID', 'VARCHAR',
]

export interface DuckdbFunction {
  name: string
  signature: string
  detail: string
}

export const DUCKDB_FUNCTIONS: DuckdbFunction[] = [
  { name: 'abs', signature: 'abs(x)', detail: 'Absolute value' },
  { name: 'any_value', signature: 'any_value(x)', detail: 'Arbitrary non-null value' },
  { name: 'approx_count_distinct', signature: 'approx_count_distinct(x)', detail: 'Approximate distinct count' },
  { name: 'arg_max', signature: 'arg_max(arg, val)', detail: 'Arg of the maximum value' },
  { name: 'arg_min', signature: 'arg_min(arg, val)', detail: 'Arg of the minimum value' },
  { name: 'array_agg', signature: 'array_agg(x)', detail: 'Collect values into a list' },
  { name: 'avg', signature: 'avg(x)', detail: 'Average' },
  { name: 'bool_and', signature: 'bool_and(x)', detail: 'True if all values are true' },
  { name: 'bool_or', signature: 'bool_or(x)', detail: 'True if any value is true' },
  { name: 'cast', signature: 'cast(x AS type)', detail: 'Convert a value to another type' },
  { name: 'ceil', signature: 'ceil(x)', detail: 'Round up' },
  { name: 'coalesce', signature: 'coalesce(x, y, …)', detail: 'First non-null value' },
  { name: 'concat', signature: 'concat(x, y, …)', detail: 'Concatenate strings' },
  { name: 'contains', signature: 'contains(string, search)', detail: 'Substring / list membership' },
  { name: 'corr', signature: 'corr(y, x)', detail: 'Pearson correlation' },
  { name: 'count', signature: 'count(*)', detail: 'Count rows or non-null values' },
  { name: 'current_date', signature: 'current_date', detail: 'Current date' },
  { name: 'date_diff', signature: 'date_diff(part, start, end)', detail: 'Difference between timestamps' },
  { name: 'date_part', signature: 'date_part(part, date)', detail: 'Extract a date / time field' },
  { name: 'date_trunc', signature: 'date_trunc(part, date)', detail: 'Truncate a timestamp' },
  { name: 'dense_rank', signature: 'dense_rank()', detail: 'Dense rank window function' },
  { name: 'epoch', signature: 'epoch(ts)', detail: 'Unix seconds for a timestamp' },
  { name: 'first', signature: 'first(x)', detail: 'First value in the group' },
  { name: 'first_value', signature: 'first_value(x)', detail: 'First value in the window' },
  { name: 'floor', signature: 'floor(x)', detail: 'Round down' },
  { name: 'format', signature: 'format(fmt, …)', detail: 'Format a string' },
  { name: 'greatest', signature: 'greatest(x, y, …)', detail: 'Largest value' },
  { name: 'histogram', signature: 'histogram(x)', detail: 'Value-count map' },
  { name: 'json_extract', signature: 'json_extract(json, path)', detail: 'Extract JSON by path' },
  { name: 'lag', signature: 'lag(x, offset := 1)', detail: 'Preceding window value' },
  { name: 'last', signature: 'last(x)', detail: 'Last value in the group' },
  { name: 'lead', signature: 'lead(x, offset := 1)', detail: 'Following window value' },
  { name: 'least', signature: 'least(x, y, …)', detail: 'Smallest value' },
  { name: 'length', signature: 'length(x)', detail: 'String or list length' },
  { name: 'list', signature: 'list(x)', detail: 'Collect values into a list' },
  { name: 'list_aggregate', signature: 'list_aggregate(list, name)', detail: 'Aggregate over a list' },
  { name: 'list_filter', signature: 'list_filter(list, lambda)', detail: 'Filter list elements' },
  { name: 'list_transform', signature: 'list_transform(list, lambda)', detail: 'Map over a list' },
  { name: 'lower', signature: 'lower(s)', detail: 'Lowercase string' },
  { name: 'max', signature: 'max(x)', detail: 'Maximum' },
  { name: 'median', signature: 'median(x)', detail: 'Median' },
  { name: 'min', signature: 'min(x)', detail: 'Minimum' },
  { name: 'mode', signature: 'mode(x)', detail: 'Most frequent value' },
  { name: 'now', signature: 'now()', detail: 'Current timestamp' },
  { name: 'nullif', signature: 'nullif(a, b)', detail: 'NULL if a equals b' },
  { name: 'ntile', signature: 'ntile(n)', detail: 'Split rows into n buckets' },
  { name: 'percentile_cont', signature: 'percentile_cont(q) WITHIN GROUP (ORDER BY x)', detail: 'Continuous percentile' },
  { name: 'printf', signature: 'printf(fmt, …)', detail: 'C-style string format' },
  { name: 'quantile', signature: 'quantile(x, q)', detail: 'Sample quantile' },
  { name: 'rank', signature: 'rank()', detail: 'Rank window function' },
  { name: 'regexp_extract', signature: 'regexp_extract(s, pattern[, group])', detail: 'Extract a regex group' },
  { name: 'regexp_matches', signature: 'regexp_matches(s, pattern)', detail: 'Regex match test' },
  { name: 'regexp_replace', signature: 'regexp_replace(s, pattern, repl)', detail: 'Regex substitution' },
  { name: 'replace', signature: 'replace(s, from, to)', detail: 'Replace all occurrences' },
  { name: 'round', signature: 'round(x[, digits])', detail: 'Round a number' },
  { name: 'row_number', signature: 'row_number()', detail: 'Sequential window index' },
  { name: 'split_part', signature: 'split_part(s, sep, n)', detail: 'Nth field after split' },
  { name: 'stddev', signature: 'stddev(x)', detail: 'Sample standard deviation' },
  { name: 'strftime', signature: 'strftime(ts, format)', detail: 'Format a timestamp' },
  { name: 'string_agg', signature: 'string_agg(s, sep)', detail: 'Concatenate grouped strings' },
  { name: 'strptime', signature: 'strptime(s, format)', detail: 'Parse a timestamp' },
  { name: 'struct_extract', signature: 'struct_extract(struct, field)', detail: 'Read a struct field' },
  { name: 'substr', signature: 'substr(s, start[, length])', detail: 'Substring' },
  { name: 'sum', signature: 'sum(x)', detail: 'Sum' },
  { name: 'trim', signature: 'trim(s)', detail: 'Strip surrounding whitespace' },
  { name: 'try_cast', signature: 'try_cast(x AS type)', detail: 'Cast, or NULL on failure' },
  { name: 'typeof', signature: 'typeof(x)', detail: 'Runtime type name' },
  { name: 'unnest', signature: 'unnest(list)', detail: 'Expand a list to rows' },
  { name: 'upper', signature: 'upper(s)', detail: 'Uppercase string' },
  { name: 'variance', signature: 'variance(x)', detail: 'Sample variance' },
]

export const duckdbLanguageConfig: languages.LanguageConfiguration = {
  comments: {
    lineComment: '--',
    blockComment: ['/*', '*/'],
  },
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')'],
  ],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
}

export const duckdbMonarch: languages.IMonarchLanguage = {
  defaultToken: '',
  tokenPostfix: '.duckdb',
  ignoreCase: true,
  keywords: DUCKDB_KEYWORDS,
  typeKeywords: DUCKDB_TYPES,
  builtinFunctions: DUCKDB_FUNCTIONS.map((item) => item.name),
  operators: [
    'ALL', 'AND', 'ANY', 'BETWEEN', 'EXISTS', 'IN', 'LIKE', 'ILIKE', 'GLOB',
    'NOT', 'OR', 'SOME', 'EXCEPT', 'INTERSECT', 'UNION', 'ANTI', 'ASOF',
    'CROSS', 'FULL', 'INNER', 'JOIN', 'LEFT', 'NATURAL', 'OUTER', 'POSITIONAL',
    'RIGHT', 'SEMI', 'IS', 'NULL', 'SIMILAR',
  ],
  tokenizer: {
    root: [
      { include: '@comments' },
      { include: '@whitespace' },
      { include: '@numbers' },
      { include: '@strings' },
      { include: '@identifiers' },
      [/[;,.]/, 'delimiter'],
      [/[()]/, '@brackets'],
      [
        /[\w$]+/,
        {
          cases: {
            '@operators': 'operator',
            '@typeKeywords': 'type',
            '@builtinFunctions': 'predefined',
            '@keywords': 'keyword',
            '@default': 'identifier',
          },
        },
      ],
      [/[<>=!%&+\-*/|~^]/, 'operator'],
    ],
    whitespace: [[/\s+/, 'white']],
    comments: [
      [/--+.*/, 'comment'],
      [/\/\*/, { token: 'comment.quote', next: '@comment' }],
    ],
    comment: [
      [/[^*/]+/, 'comment'],
      [/\*\//, { token: 'comment.quote', next: '@pop' }],
      [/./, 'comment'],
    ],
    numbers: [
      [/0[xX][0-9a-fA-F]+/, 'number'],
      [/((\d+(\.\d*)?)|(\.\d+))([eE][-+]?\d+)?/, 'number'],
    ],
    strings: [
      [/'/, { token: 'string', next: '@string' }],
    ],
    string: [
      [/[^']+/, 'string'],
      [/''/, 'string'],
      [/'/, { token: 'string', next: '@pop' }],
    ],
    identifiers: [
      [/"/, { token: 'identifier.quote', next: '@quoted' }],
    ],
    quoted: [
      [/[^"]+/, 'identifier'],
      [/""/, 'identifier'],
      [/"/, { token: 'identifier.quote', next: '@pop' }],
    ],
  },
}
