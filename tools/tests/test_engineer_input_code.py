"""Original patch code is legal; executable fixture data is not."""
import unittest
from unittest.mock import patch

from tools import native_oracle as native
from tools.spatial_oracle import engineer_repair_admission as owner
from tools.tests.pe_fixture import pe_image


class EngineerInputCodeTests(unittest.TestCase):
    def test_patch_text_is_admitted_but_data_and_zero_filled_tail_are_rejected(self):
        image = pe_image([
            (0x82E000, 0x200, b'\x90' * 0x1000, 0x1100, 0x60000020),
            (0x700000, 0x1200, b'\x90' * 0x20, 0x20, 0xC0000040),
        ])
        fixture = owner.EngineerInputs.__new__(owner.EngineerInputs)
        with patch.object(native, 'image_bytes', return_value=image), \
                patch.object(owner.bridge_child_sound.Sound, 'hook') as parent:
            fixture.hook(None, 0xC2EF7B, 3, None)
            parent.assert_called_once_with(None, 0xC2EF7B, 3, None)
            for pc in (0xB00010, 0xC2F005):
                with self.subTest(pc=hex(pc)), self.assertRaisesRegex(AssertionError, 'non_native_input_code'):
                    fixture.hook(None, pc, 1, None)
            self.assertEqual(parent.call_count, 1)


if __name__ == '__main__':
    unittest.main()
