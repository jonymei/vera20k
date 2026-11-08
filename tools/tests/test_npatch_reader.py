"""Shared NP setup uses the existing allocator and preserves caller CPU state."""
import unittest
from unittest.mock import patch

from unicorn import Uc, UC_ARCH_X86, UC_MODE_32
from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_EBX, UC_X86_REG_ESP, UC_X86_REG_EIP

from tools.rules_oracle.bridge_anim_inputs import Reader
from tools.rules_oracle.bridge_anim_lists import HEAP
from tools.spatial_oracle.map_queries import dwords


class NpatchReaderTests(unittest.TestCase):
    def reader(self):
        m = Reader.__new__(Reader)
        m.u = Uc(UC_ARCH_X86, UC_MODE_32)
        m.u.mem_map(HEAP, 0x400000)
        m.arena_size, m.cursor, m.npatch = 0x400000, HEAP + 0x10000, True
        return m

    def test_crt_malloc_and_operator_new_share_one_cursor_and_return_abi(self):
        m = self.reader()
        sp = HEAP + 0x1000
        for entry, expected in ((0x7C9430, HEAP + 0x10000),
                                (0x7C8E17, HEAP + 0x10020)):
            m.u.mem_write(sp, dwords(0x12345678, 17))
            m.u.reg_write(UC_X86_REG_ESP, sp)
            m.hook(m.u, entry, 1, None)
            self.assertEqual(m.u.reg_read(UC_X86_REG_EAX), expected)
            self.assertEqual(m.u.reg_read(UC_X86_REG_EIP), 0x12345678)
            self.assertEqual(m.u.reg_read(UC_X86_REG_ESP), sp + 4)
        self.assertEqual(m.cursor, HEAP + 0x10040)

    def test_setup_failure_does_not_leak_registers_into_composed_caller(self):
        m = self.reader()
        def fail(*args):
            m.u.reg_write(UC_X86_REG_EBX, 0xBAD)
            raise RuntimeError('native setup stopped')
        for operation in (m.initialize_npatch_rules, lambda: m.read_npatch_rules(HEAP)):
            m.u.reg_write(UC_X86_REG_EBX, 123)
            with patch.object(m, 'invoke', side_effect=fail):
                with self.assertRaisesRegex(RuntimeError, 'native setup stopped'):
                    operation()
            self.assertEqual(m.u.reg_read(UC_X86_REG_EBX), 123)


if __name__ == '__main__':
    unittest.main()
