"""Sparse fixture state must not overwrite original patch instructions."""
import struct
import unittest
from types import SimpleNamespace
from unittest.mock import Mock, patch

from unicorn import Uc, UC_ARCH_X86, UC_MODE_32

from tools.native_oracle import SCRATCH
from tools.spatial_oracle import unit_source_scatter as owner
from tools.spatial_oracle import engineer_repair_admission as joined
from tools.spatial_oracle.map_queries import dwords


class SourceFixtureMemoryTests(unittest.TestCase):
    def test_sparse_cells_preserve_patch_code_and_remain_addressable(self):
        u = Uc(UC_ARCH_X86, UC_MODE_32)
        u.mem_map(0x400000, 0x900000)
        u.mem_map(SCRATCH, 0x10000)
        patch_code = b'\x90\xc3' * 0x800
        u.mem_write(0xC27000, patch_code)
        u.mem_write(0x7F5C70 + 0x1AC, dwords(0x73F0A0))
        u.mem_write(0x7F5C70 + 0x480, dwords(0x741970))
        read32 = lambda p: struct.unpack('<I', u.mem_read(p, 4))[0]
        with patch.object(owner, 'make_fixture', return_value=(u, Mock(), read32)):
            owner.make_source_fixture({})
        self.assertEqual(bytes(u.mem_read(0xC27000, len(patch_code))), patch_code)
        table = read32(owner.MAP + 0x13C)
        cell = read32(table + (10 * 512 + 11) * 4)
        self.assertEqual(bytes(u.mem_read(cell + 0x24, 4)), struct.pack('<hh', 11, 10))
        self.assertEqual(read32(table + (100 * 512 + 100) * 4), 0)

    def test_engineer_join_preserves_the_parent_map_table_binding(self):
        u = Uc(UC_ARCH_X86, UC_MODE_32)
        u.mem_map(0x400000, 0x900000)
        u.mem_map(SCRATCH, 0x10000)
        u.mem_map(joined.HOUSE & ~0xFFF, 0x10000)
        u.mem_write(owner.MAP + 0x13C, dwords(owner.TABLE, 0x40000))
        u.mem_write(0xAC13C8, dwords(104))
        u.mem_write(0xAC13BC, dwords(416))
        read32 = lambda p: struct.unpack('<I', u.mem_read(p, 4))[0]
        def parent(fixture, *args, **kwargs):
            fixture.u, fixture.read32, fixture.heap = u, read32, SCRATCH
        class PrefixComplete(Exception):
            pass
        # Exercise the production composition prefix, not the later audio or
        # gameplay setup. The parent and CRT effects are supplied test state.
        original_map = Uc.mem_map
        def stop_at_navigation(vm, address, size, *args, **kwargs):
            if address == 0x31000000:
                raise PrefixComplete
            return original_map(vm, address, size, *args, **kwargs)
        with patch.object(joined.construction.JoinedFixture, '__init__', parent), \
                patch.object(joined.construction, 'invoke'), \
                patch.object(Uc, 'mem_map', stop_at_navigation):
            with self.assertRaises(PrefixComplete):
                joined.EngineerJoinedFixture(SimpleNamespace(u=u, country=0))
        self.assertEqual(read32(owner.MAP + 0x13C), owner.TABLE)


if __name__ == '__main__':
    unittest.main()
