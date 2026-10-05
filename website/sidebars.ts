import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

/**
 * One hand-written sidebar: what Canon is and a first run, the ideas behind it, the generated
 * reference, the generated worked example, and where the project stands.
 */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Start here',
      collapsed: false,
      items: ['index', 'getting-started'],
    },
    {
      type: 'category',
      label: 'Concepts',
      collapsed: false,
      items: [
        'concepts/protocols',
        'concepts/three-valued-truth',
        'concepts/evidence-and-revisions',
        'concepts/evaluation',
        'concepts/assertions',
      ],
    },
    {
      type: 'category',
      label: 'Reference',
      collapsed: false,
      items: [
        'reference/cli',
        'reference/protocol',
        'reference/documents',
        'reference/evaluation',
        'reference/validation',
        'reference/canon-ir',
        'reference/conformance',
        'reference/assertions',
      ],
    },
    {
      type: 'category',
      label: 'Examples',
      collapsed: false,
      items: ['reference/investigation-example'],
    },
    {
      type: 'category',
      label: 'Project status',
      collapsed: false,
      items: ['status/where-this-stands'],
    },
  ],
};

export default sidebars;
