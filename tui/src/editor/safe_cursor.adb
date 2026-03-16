-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Safe_Cursor - SPARK-verified cursor management (body)

pragma SPARK_Mode (On);

package body Editor.Safe_Cursor is

   -------------------
   -- Has_Selection --
   -------------------

   function Has_Selection (Cursor : Cursor_State) return Boolean is
   begin
      return Cursor.Sel_Anchor > 0 and Cursor.Sel_Anchor /= Cursor.Index;
   end Has_Selection;

   ---------------------
   -- Selection_Start --
   ---------------------

   function Selection_Start (Cursor : Cursor_State) return Buffer_Index is
   begin
      if Cursor.Index < Cursor.Sel_Anchor then
         return Cursor.Index;
      else
         return Cursor.Sel_Anchor;
      end if;
   end Selection_Start;

   -------------------
   -- Selection_End --
   -------------------

   function Selection_End (Cursor : Cursor_State) return Buffer_Index is
   begin
      if Cursor.Index > Cursor.Sel_Anchor then
         return Cursor.Index;
      else
         return Cursor.Sel_Anchor;
      end if;
   end Selection_End;

   ----------------
   -- Move_Right --
   ----------------

   procedure Move_Right (Cursor : in Out Cursor_State; Buffer : Safe_Buffer) is
      Buf_Len : constant Buffer_Index := Length (Buffer);
   begin
      if Cursor.Index < Buf_Len then
         Cursor.Index := Cursor.Index + 1;

         --  Update line/column
         if Cursor.Index > 0 and then
            Element (Buffer, Cursor.Index) = ASCII.LF
         then
            Cursor.Line := Cursor.Line + 1;
            Cursor.Column := 1;
         else
            Cursor.Column := Cursor.Column + 1;
         end if;

         Cursor.Pref_Col := Cursor.Column;
      end if;
   end Move_Right;

   ---------------
   -- Move_Left --
   ---------------

   procedure Move_Left (Cursor : in Out Cursor_State; Buffer : Safe_Buffer) is
   begin
      if Cursor.Index > 0 then
         --  Check if we're crossing a line boundary
         if Element (Buffer, Cursor.Index) = ASCII.LF then
            Cursor.Line := Cursor.Line - 1;
            --  Calculate column on previous line
            Cursor.Column := Line_Length (Buffer, Cursor.Line) + 1;
         else
            Cursor.Column := Cursor.Column - 1;
         end if;

         Cursor.Index := Cursor.Index - 1;
         Cursor.Pref_Col := Cursor.Column;
      end if;
   end Move_Left;

   -------------
   -- Move_Up --
   -------------

   procedure Move_Up (Cursor : in Out Cursor_State; Buffer : Safe_Buffer) is
      Prev_Line_Start : Buffer_Index;
      Prev_Line_Len   : Natural;
      Target_Col      : Positive;
   begin
      if Cursor.Line > 1 then
         --  Calculate position on previous line
         Prev_Line_Start := Line_Start (Buffer, Cursor.Line - 1);
         Prev_Line_Len := Line_Length (Buffer, Cursor.Line - 1);

         --  Use preferred column, clamped to line length
         if Cursor.Pref_Col <= Prev_Line_Len then
            Target_Col := Cursor.Pref_Col;
         else
            Target_Col := (if Prev_Line_Len > 0 then Prev_Line_Len else 1);
         end if;

         Cursor.Index := Prev_Line_Start + Target_Col - 1;
         Cursor.Line := Cursor.Line - 1;
         Cursor.Column := Target_Col;
         --  Keep Pref_Col unchanged
      end if;
   end Move_Up;

   ---------------
   -- Move_Down --
   ---------------

   procedure Move_Down (Cursor : in Out Cursor_State; Buffer : Safe_Buffer) is
      Total_Lines     : constant Positive := Line_Count (Buffer);
      Next_Line_Start : Buffer_Index;
      Next_Line_Len   : Natural;
      Target_Col      : Positive;
   begin
      if Cursor.Line < Total_Lines then
         --  Calculate position on next line
         Next_Line_Start := Line_Start (Buffer, Cursor.Line + 1);
         Next_Line_Len := Line_Length (Buffer, Cursor.Line + 1);

         --  Use preferred column, clamped to line length
         if Cursor.Pref_Col <= Next_Line_Len then
            Target_Col := Cursor.Pref_Col;
         else
            Target_Col := (if Next_Line_Len > 0 then Next_Line_Len else 1);
         end if;

         Cursor.Index := Next_Line_Start + Target_Col - 1;
         Cursor.Line := Cursor.Line + 1;
         Cursor.Column := Target_Col;
         --  Keep Pref_Col unchanged
      end if;
   end Move_Down;

   ------------------------
   -- Move_To_Line_Start --
   ------------------------

   procedure Move_To_Line_Start (Cursor : in Out Cursor_State; Buffer : Safe_Buffer) is
      Start_Idx : constant Buffer_Index := Line_Start (Buffer, Cursor.Line);
   begin
      if Start_Idx > 0 then
         Cursor.Index := Start_Idx - 1;  -- Position before first char
      else
         Cursor.Index := 0;
      end if;
      Cursor.Column := 1;
      Cursor.Pref_Col := 1;
   end Move_To_Line_Start;

   ----------------------
   -- Move_To_Line_End --
   ----------------------

   procedure Move_To_Line_End (Cursor : in Out Cursor_State; Buffer : Safe_Buffer) is
      Line_Len  : constant Natural := Line_Length (Buffer, Cursor.Line);
      Start_Idx : constant Buffer_Index := Line_Start (Buffer, Cursor.Line);
   begin
      Cursor.Index := Start_Idx + Line_Len - 1;
      Cursor.Column := (if Line_Len > 0 then Line_Len else 1);
      Cursor.Pref_Col := Cursor.Column;
   end Move_To_Line_End;

   --------------------------
   -- Move_To_Buffer_Start --
   --------------------------

   procedure Move_To_Buffer_Start (Cursor : in Out Cursor_State) is
   begin
      Cursor.Index := 0;
      Cursor.Line := 1;
      Cursor.Column := 1;
      Cursor.Pref_Col := 1;
   end Move_To_Buffer_Start;

   ------------------------
   -- Move_To_Buffer_End --
   ------------------------

   procedure Move_To_Buffer_End (Cursor : in Out Cursor_State; Buffer : Safe_Buffer) is
   begin
      Cursor.Index := Length (Buffer);
      Cursor.Line := Line_Count (Buffer);
      Cursor.Column := Line_Length (Buffer, Cursor.Line) + 1;
      Cursor.Pref_Col := Cursor.Column;
   end Move_To_Buffer_End;

   -------------
   -- Move_To --
   -------------

   procedure Move_To (Cursor : in Out Cursor_State;
                      Buffer : Safe_Buffer;
                      Target : Position) is
      Total_Lines : constant Positive := Line_Count (Buffer);
      Line_Num    : Positive;
      Line_Len    : Natural;
      Target_Col  : Positive;
      Start_Idx   : Buffer_Index;
   begin
      --  Clamp line to valid range
      if Target.Line <= Total_Lines then
         Line_Num := Target.Line;
      else
         Line_Num := Total_Lines;
      end if;

      --  Get line info
      Start_Idx := Line_Start (Buffer, Line_Num);
      Line_Len := Line_Length (Buffer, Line_Num);

      --  Clamp column to valid range
      if Target.Column <= Line_Len + 1 then
         Target_Col := Target.Column;
      else
         Target_Col := (if Line_Len > 0 then Line_Len + 1 else 1);
      end if;

      --  Update cursor
      Cursor.Index := Start_Idx + Target_Col - 2;
      if Cursor.Index > Length (Buffer) then
         Cursor.Index := Length (Buffer);
      end if;
      Cursor.Line := Line_Num;
      Cursor.Column := Target_Col;
      Cursor.Pref_Col := Target_Col;
   end Move_To;

   ---------------------
   -- Start_Selection --
   ---------------------

   procedure Start_Selection (Cursor : in Out Cursor_State) is
   begin
      Cursor.Sel_Anchor := Cursor.Index;
   end Start_Selection;

   ---------------------
   -- Clear_Selection --
   ---------------------

   procedure Clear_Selection (Cursor : in Out Cursor_State) is
   begin
      Cursor.Sel_Anchor := 0;
   end Clear_Selection;

   ----------------------
   -- Extend_Selection --
   ----------------------

   procedure Extend_Selection (Cursor : in Out Cursor_State) is
   begin
      --  Selection already started, anchor stays, index moves
      null;
   end Extend_Selection;

   -------------------
   -- Sync_Position --
   -------------------

   procedure Sync_Position (Cursor : in Out Cursor_State; Buffer : Safe_Buffer) is
      Buf_Len : constant Buffer_Index := Length (Buffer);
      Pos     : Position;
   begin
      --  Clamp index to buffer bounds
      if Cursor.Index > Buf_Len then
         Cursor.Index := Buf_Len;
      end if;

      --  Recalculate line/column from index
      Pos := Index_To_Position (Buffer, Cursor.Index);
      Cursor.Line := Pos.Line;
      Cursor.Column := Pos.Column;

      --  Clamp selection anchor
      if Cursor.Sel_Anchor > Buf_Len then
         Cursor.Sel_Anchor := Buf_Len;
      end if;
   end Sync_Position;

end Editor.Safe_Cursor;
