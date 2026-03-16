-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Formatrix TUI - Application state and logic

with Terminal_Interface.Curses;
with Ada.Strings.Unbounded;

package Formatrix_TUI.App is

   use Ada.Strings.Unbounded;
   use Terminal_Interface.Curses;

   --  Document format types
   type Document_Format is
     (Plain_Text, Markdown, AsciiDoc, Djot, Org_Mode, RST, Typst);

   --  Focus areas for keyboard input routing
   type Focus_Area is (Editor, Tabs, Dialog);

   --  Dialog types
   type Dialog_Type is (None, Save_As, Open_File, Quit_Confirm, Error_Msg);

   --  Application state
   type App_State is record
      --  Current format tab (0-6)
      Active_Tab      : Natural := 1;  -- Default to Markdown

      --  Document content (simple buffer for now)
      Content         : Unbounded_String := Null_Unbounded_String;

      --  File path if saved
      File_Path       : Unbounded_String := Null_Unbounded_String;

      --  Original format when file was opened
      Original_Format : Document_Format := Markdown;

      --  Whether document has unsaved changes
      Modified        : Boolean := False;

      --  Current focus area
      Focus           : Focus_Area := Editor;

      --  Current dialog
      Current_Dialog  : Dialog_Type := None;
      Dialog_Input    : Unbounded_String := Null_Unbounded_String;

      --  Status message
      Status_Message  : Unbounded_String := Null_Unbounded_String;

      --  Cursor position
      Cursor_Row      : Natural := 0;
      Cursor_Col      : Natural := 0;

      --  Should quit flag
      Should_Quit     : Boolean := False;
   end record;

   --  Get format label for tab display
   function Format_Label (Format : Document_Format) return String;

   --  Get all formats
   All_Formats : constant array (0 .. 6) of Document_Format :=
     (Plain_Text, Markdown, AsciiDoc, Djot, Org_Mode, RST, Typst);

   --  Handle keyboard input
   procedure Handle_Key (State : in out App_State; Key : Key_Code);

   --  Load a file
   procedure Load_File (State : in out App_State; Path : String);

   --  Save current file
   procedure Save_File (State : in Out App_State);

   --  Switch to next/previous format tab
   procedure Next_Tab (State : in out App_State);
   procedure Prev_Tab (State : in Out App_State);

end Formatrix_TUI.App;
