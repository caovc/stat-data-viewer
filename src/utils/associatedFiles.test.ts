import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import { consumePendingOpenPaths } from './associatedFiles.ts'

describe('consumePendingOpenPaths', () => {
  it('opens every pending launch path in order, then sees an empty queue', async () => {
    const opened: string[] = []
    const queue = [
      '/tmp/adsl.sas7bdat',
      '/tmp/adae.xpt',
      '/tmp/demo.xport',
      '/tmp/survey.sav',
      '/tmp/survey.zsav',
      '/tmp/legacy.por',
      '/tmp/study.dta',
    ]

    await consumePendingOpenPaths(
      async () => queue.splice(0),
      async (path) => {
        opened.push(path)
      },
    )

    assert.deepEqual(opened, [
      '/tmp/adsl.sas7bdat',
      '/tmp/adae.xpt',
      '/tmp/demo.xport',
      '/tmp/survey.sav',
      '/tmp/survey.zsav',
      '/tmp/legacy.por',
      '/tmp/study.dta',
    ])
    assert.deepEqual(
      await consumePendingOpenPaths(
        async () => queue.splice(0),
        async (path) => {
          opened.push(path)
        },
      ),
      [],
    )
    assert.deepEqual(opened, [
      '/tmp/adsl.sas7bdat',
      '/tmp/adae.xpt',
      '/tmp/demo.xport',
      '/tmp/survey.sav',
      '/tmp/survey.zsav',
      '/tmp/legacy.por',
      '/tmp/study.dta',
    ])
  })
})
