"""Retained-plane input bounds and compact native-query projection."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.spatial_oracle import bridge_connectivity


class RetainedPlaneReportTests(unittest.TestCase):
    def source(self):
        return dict(name='fixture', map='fixture.map', width=2, height=2,
                    native_size=[1, 1], classes=[0, 4, 0, 4], levels=[0] * 4,
                    records=[], records_match_bridge_authority=True,
                    cells=[dict(coord=[0, 0]), dict(coord=[1, 1])])

    def report(self, source):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root) / 'plane.json'
            path.write_text(json.dumps(source), encoding='utf-8')
            return bridge_connectivity.retained_plane_report(path)

    def test_compact_queries_use_water_row_ten_and_exported_map_name(self):
        # Synthetic transport inputs, not native execution goldens.
        rows = [[0, 1, 1] for _ in range(13)]
        rows[10] = [0, 27, 42]
        with patch.object(bridge_connectivity, 'execute', return_value=dict(
                base_ids=[1, 1, 2, 2], raw_rows=rows)) as execute:
            result = self.report(self.source())
        self.assertEqual(result['input'], dict(map='fixture.map', size=[1, 1]))
        self.assertEqual(result['queries'], [dict(cell=[0, 0], water_zone=27),
                                             dict(cell=[1, 1], water_zone=42)])
        self.assertNotIn('classes', result)
        self.assertNotIn('base_ids', result)
        self.assertEqual(execute.call_args.args[0]['size'], [1, 1])

    def test_bridge_inputs_are_rejected_before_native_execution(self):
        source = self.source()
        source['records'] = [dict(a=[0, 0], b=[1, 1], active=True, kind=0)]
        with patch.object(bridge_connectivity, 'execute') as execute:
            with self.assertRaises(AssertionError):
                self.report(source)
        execute.assert_not_called()


if __name__ == '__main__':
    unittest.main()
