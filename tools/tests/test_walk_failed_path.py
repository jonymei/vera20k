"""Sparse fixture data must not overwrite an extended executable's code."""
import unittest
from unittest.mock import patch

from unicorn.x86_const import UC_X86_REG_ESP

from tools.spatial_oracle import walk_failed_path as owner


class InspectionComplete(Exception):
    pass


class WalkFixtureTests(unittest.TestCase):
    def test_cell_table_preserves_patch_code_before_the_first_setter(self):
        sentinel_address = 0xC2BD1A
        sentinel = b'NP_CODE_MUST_SURVIVE'

        def synthetic_image(uc):
            uc.mem_map(0x400000, 0xC00000)
            uc.mem_write(sentinel_address, sentinel)

        def inspect(uc, begin, _end, **_kwargs):
            if begin == 0x75AA90:
                # Supply the unused constructor return; no game code runs.
                uc.reg_write(UC_X86_REG_ESP, uc.reg_read(UC_X86_REG_ESP) + 4)
                return
            self.assertEqual(begin, 0x51AA40)
            self.assertEqual(bytes(uc.mem_read(sentinel_address, len(sentinel))), sentinel)
            self.assertEqual(bytes(uc.mem_read(owner.TABLE + (10 * 512 + 11) * 4, 4)),
                             owner.dwords(owner.CELL + 0x200))
            raise InspectionComplete

        with patch.object(owner, 'load_image', synthetic_image), \
                patch.object(owner, 'run_checked', inspect):
            with self.assertRaises(InspectionComplete):
                owner.query({})


if __name__ == '__main__':
    unittest.main()
