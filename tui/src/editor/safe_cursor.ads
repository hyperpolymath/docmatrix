-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Safe_Cursor - SPARK-verified cursor management
--
-- This module provides a statically-verified cursor for text editing with:
-- - Guaranteed bounds checking
-- - Movement constraints proven at compile time
-- - No possibility of cursor escaping buffer bounds

pragma SPARK_Mode (On);

with Editor.Safe_Buffer; use Editor.Safe_Buffer;

package Editor.Safe_Cursor is

   --  Cursor state
   type Cursor_State is record
      --  Current buffer index (0 = before first character)
      Index      : Buffer_Index := 0;
      --  Current line number (1-indexed)
      Line       : Positive := 1;
      --  Current column number (1-indexed)
      Column     : Positive := 1;
      --  Preferred column (remembered across vertical movements)
      Pref_Col   : Positive := 1;
      --  Selection anchor (0 = no selection)
      Sel_Anchor : Buffer_Index := 0;
   end record;

   --  Check if cursor has a selection
   function Has_Selection (Cursor : Cursor_State) return Boolean
     with Global => null,
          Post   => Has_Selection'Result = (Cursor.Sel_Anchor > 0 and
                                            Cursor.Sel_Anchor /= Cursor.Index);

   --  Get selection start index
   function Selection_Start (Cursor : Cursor_State) return Buffer_Index
     with Global => null,
          Pre    => Has_Selection (Cursor);

   --  Get selection end index
   function Selection_End (Cursor : Cursor_State) return Buffer_Index
     with Global => null,
          Pre    => Has_Selection (Cursor);

   --  Move cursor right by one character
   procedure Move_Right (Cursor : in Out Cursor_State; Buffer : Safe_Buffer)
     with Global => null,
          Post   => Cursor.Index <= Length (Buffer);

   --  Move cursor left by one character
   procedure Move_Left (Cursor : in Out Cursor_State; Buffer : Safe_Buffer)
     with Global => null,
          Post   => Cursor.Index <= Length (Buffer);

   --  Move cursor up by one line
   procedure Move_Up (Cursor : in Out Cursor_State; Buffer : Safe_Buffer)
     with Global => null,
          Post   => Cursor.Index <= Length (Buffer) and Cursor.Line >= 1;

   --  Move cursor down by one line
   procedure Move_Down (Cursor : in Out Cursor_State; Buffer : Safe_Buffer)
     with Global => null,
          Post   => Cursor.Index <= Length (Buffer);

   --  Move cursor to start of current line
   procedure Move_To_Line_Start (Cursor : in Out Cursor_State; Buffer : Safe_Buffer)
     with Global => null,
          Post   => Cursor.Index <= Length (Buffer) and Cursor.Column = 1;

   --  Move cursor to end of current line
   procedure Move_To_Line_End (Cursor : in Out Cursor_State; Buffer : Safe_Buffer)
     with Global => null,
          Post   => Cursor.Index <= Length (Buffer);

   --  Move cursor to start of buffer
   procedure Move_To_Buffer_Start (Cursor : in Out Cursor_State)
     with Global => null,
          Post   => Cursor.Index = 0 and Cursor.Line = 1 and Cursor.Column = 1;

   --  Move cursor to end of buffer
   procedure Move_To_Buffer_End (Cursor : in Out Cursor_State; Buffer : Safe_Buffer)
     with Global => null,
          Post   => Cursor.Index = Length (Buffer);

   --  Move cursor to specific position (clamped to buffer bounds)
   procedure Move_To (Cursor : in Out Cursor_State;
                      Buffer : Safe_Buffer;
                      Target : Position)
     with Global => null,
          Post   => Cursor.Index <= Length (Buffer);

   --  Start selection at current position
   procedure Start_Selection (Cursor : in Out Cursor_State)
     with Global => null,
          Post   => Cursor.Sel_Anchor = Cursor.Index'Old;

   --  Clear selection
   procedure Clear_Selection (Cursor : in Out Cursor_State)
     with Global => null,
          Post   => Cursor.Sel_Anchor = 0 and not Has_Selection (Cursor);

   --  Extend selection to current position
   procedure Extend_Selection (Cursor : in Out Cursor_State)
     with Global => null;

   --  Synchronize cursor position with buffer
   --  (call after buffer modifications)
   procedure Sync_Position (Cursor : in Out Cursor_State; Buffer : Safe_Buffer)
     with Global => null,
          Post   => Cursor.Index <= Length (Buffer);

end Editor.Safe_Cursor;
