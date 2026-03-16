-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Formatrix TUI - UI rendering (body)

with Ada.Strings.Unbounded; use Ada.Strings.Unbounded;

package body Formatrix_TUI.UI is

   procedure Setup_Colors is
   begin
      Init_Pair (Tab_Inactive, White, Black);
      Init_Pair (Tab_Active, Black, Cyan);
      Init_Pair (Editor_Normal, White, Black);
      Init_Pair (Status_Bar, White, Blue);
      Init_Pair (Dialog_Border, Cyan, Black);
   end Setup_Colors;

   procedure Render (State : Formatrix_TUI.App.App_State) is
   begin
      Erase;

      Render_Tabs (State);
      Render_Editor (State);
      Render_Status (State);

      if State.Current_Dialog /= Formatrix_TUI.App.None then
         Render_Dialog (State);
      end if;
   end Render;

   procedure Render_Tabs (State : Formatrix_TUI.App.App_State) is
      Col : Column_Position := 0;
   begin
      Move_Cursor (Line => 0, Column => 0);

      for I in Formatrix_TUI.App.All_Formats'Range loop
         declare
            Label : constant String :=
              " " & Formatrix_TUI.App.Format_Label
                      (Formatrix_TUI.App.All_Formats (I)) & " ";
         begin
            if I = State.Active_Tab then
               Set_Color (Tab_Active);
               Switch_Character_Attribute (Attr => (Bold_Character => True, others => False),
                                           On => True);
            else
               Set_Color (Tab_Inactive);
            end if;

            Add (Str => Label);

            Set_Color (Editor_Normal);
            Switch_Character_Attribute (Attr => (Bold_Character => True, others => False),
                                        On => False);

            Add (Ch => '|');
            Col := Col + Column_Position (Label'Length) + 1;
         end;
      end loop;

      --  Draw separator line
      Move_Cursor (Line => 1, Column => 0);
      for I in 0 .. Columns - 1 loop
         Add (Ch => '-');
      end loop;
   end Render_Tabs;

   procedure Render_Editor (State : Formatrix_TUI.App.App_State) is
      Start_Line : constant Line_Position := 2;
      End_Line   : constant Line_Position := Lines - 2;
      Content    : constant String := To_String (State.Content);
      Line_Num   : Line_Position := Start_Line;
      Col        : Column_Position := 0;
   begin
      Set_Color (Editor_Normal);

      --  Clear editor area
      for L in Start_Line .. End_Line loop
         Move_Cursor (Line => L, Column => 0);
         Clear_To_End_Of_Line;
      end loop;

      --  Draw content
      Move_Cursor (Line => Start_Line, Column => 0);

      for C of Content loop
         if C = ASCII.LF then
            Line_Num := Line_Num + 1;
            Col := 0;
            if Line_Num <= End_Line then
               Move_Cursor (Line => Line_Num, Column => 0);
            end if;
         elsif Line_Num <= End_Line and Col < Columns then
            Add (Ch => C);
            Col := Col + 1;
         end if;
      end loop;

      --  Draw border
      Move_Cursor (Line => Start_Line - 1, Column => 0);
      declare
         Title : constant String :=
           (if Length (State.File_Path) > 0
            then " " & To_String (State.File_Path) & " "
            else " [untitled] ");
         Modified_Marker : constant String :=
           (if State.Modified then "[+] " else "");
      begin
         Add (Str => Modified_Marker & Title);
      end;
   end Render_Editor;

   procedure Render_Status (State : Formatrix_TUI.App.App_State) is
      Status_Line : constant Line_Position := Lines - 1;
   begin
      Move_Cursor (Line => Status_Line, Column => 0);
      Set_Color (Status_Bar);

      --  Left: status message or keybinding hints
      if Length (State.Status_Message) > 0 then
         Add (Str => To_String (State.Status_Message));
      else
         Add (Str => "Ctrl+S:Save  Ctrl+O:Open  Ctrl+Q:Quit  Tab:Next");
      end if;

      --  Pad to end of line
      Clear_To_End_Of_Line;

      --  Right: format indicator
      declare
         Format_Str : constant String :=
           Formatrix_TUI.App.Format_Label
             (Formatrix_TUI.App.All_Formats (State.Active_Tab));
      begin
         Move_Cursor (Line => Status_Line,
                      Column => Columns - Column_Position (Format_Str'Length) - 1);
         Add (Str => Format_Str);
      end;

      Set_Color (Editor_Normal);
   end Render_Status;

   procedure Render_Dialog (State : Formatrix_TUI.App.App_State) is
      Dialog_Width  : constant Column_Position := 50;
      Dialog_Height : constant Line_Position := 5;
      Start_Col     : constant Column_Position := (Columns - Dialog_Width) / 2;
      Start_Line    : constant Line_Position := (Lines - Dialog_Height) / 2;
   begin
      Set_Color (Dialog_Border);

      --  Draw dialog box
      for L in Start_Line .. Start_Line + Dialog_Height loop
         Move_Cursor (Line => L, Column => Start_Col);
         if L = Start_Line or L = Start_Line + Dialog_Height then
            for C in 0 .. Dialog_Width loop
               Add (Ch => '-');
            end loop;
         else
            Add (Ch => '|');
            for C in 1 .. Dialog_Width - 1 loop
               Add (Ch => ' ');
            end loop;
            Add (Ch => '|');
         end if;
      end loop;

      --  Dialog title and content
      Move_Cursor (Line => Start_Line + 1, Column => Start_Col + 2);

      case State.Current_Dialog is
         when Formatrix_TUI.App.Save_As =>
            Add (Str => "Save As:");
            Move_Cursor (Line => Start_Line + 2, Column => Start_Col + 2);
            Add (Str => To_String (State.Dialog_Input) & "_");

         when Formatrix_TUI.App.Open_File =>
            Add (Str => "Open File:");
            Move_Cursor (Line => Start_Line + 2, Column => Start_Col + 2);
            Add (Str => To_String (State.Dialog_Input) & "_");

         when Formatrix_TUI.App.Quit_Confirm =>
            Add (Str => "Unsaved changes! Quit? (y/n)");

         when Formatrix_TUI.App.Error_Msg =>
            Add (Str => "Error:");
            Move_Cursor (Line => Start_Line + 2, Column => Start_Col + 2);
            Add (Str => To_String (State.Status_Message));

         when others =>
            null;
      end case;

      Move_Cursor (Line => Start_Line + 4, Column => Start_Col + 2);
      Add (Str => "Enter: Confirm  Esc: Cancel");

      Set_Color (Editor_Normal);
   end Render_Dialog;

end Formatrix_TUI.UI;
