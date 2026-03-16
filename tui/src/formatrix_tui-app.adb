-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Formatrix TUI - Application state and logic (body)

with Ada.Text_IO;
with Ada.Directories;

package body Formatrix_TUI.App is

   function Format_Label (Format : Document_Format) return String is
   begin
      case Format is
         when Plain_Text => return "TXT";
         when Markdown   => return "MD";
         when AsciiDoc   => return "ADOC";
         when Djot       => return "DJOT";
         when Org_Mode   => return "ORG";
         when RST        => return "RST";
         when Typst      => return "TYP";
      end case;
   end Format_Label;

   procedure Handle_Key (State : in Out App_State; Key : Key_Code) is
      use Terminal_Interface.Curses;
   begin
      --  Handle dialog input first
      if State.Current_Dialog /= None then
         case Key is
            when Character'Pos (ASCII.ESC) =>
               State.Current_Dialog := None;
               State.Focus := Editor;

            when Character'Pos (ASCII.LF) | Key_Enter =>
               --  Confirm dialog action
               case State.Current_Dialog is
                  when Save_As =>
                     State.File_Path := State.Dialog_Input;
                     Save_File (State);
                  when Open_File =>
                     Load_File (State, To_String (State.Dialog_Input));
                  when Quit_Confirm =>
                     State.Should_Quit := True;
                  when others =>
                     null;
               end case;
               State.Current_Dialog := None;
               State.Focus := Editor;

            when others =>
               --  Add character to dialog input
               if Key >= 32 and Key < 127 then
                  Append (State.Dialog_Input, Character'Val (Key));
               elsif Key = Key_Backspace and
                     Length (State.Dialog_Input) > 0
               then
                  Delete (State.Dialog_Input,
                          Length (State.Dialog_Input),
                          Length (State.Dialog_Input));
               end if;
         end case;
         return;
      end if;

      --  Global keybindings
      case Key is
         --  Ctrl+Q: Quit
         when Character'Pos (ASCII.DC1) =>  -- Ctrl+Q
            if State.Modified then
               State.Current_Dialog := Quit_Confirm;
               State.Focus := Dialog;
            else
               State.Should_Quit := True;
            end if;

         --  Ctrl+S: Save
         when Character'Pos (ASCII.DC3) =>  -- Ctrl+S
            if Length (State.File_Path) > 0 then
               Save_File (State);
            else
               State.Current_Dialog := Save_As;
               State.Dialog_Input := Null_Unbounded_String;
               State.Focus := Dialog;
            end if;

         --  Ctrl+O: Open
         when Character'Pos (ASCII.SI) =>  -- Ctrl+O
            State.Current_Dialog := Open_File;
            State.Dialog_Input := Null_Unbounded_String;
            State.Focus := Dialog;

         --  Tab: Next format
         when Character'Pos (ASCII.HT) =>
            Next_Tab (State);

         --  Shift+Tab (backtab): Previous format
         when Key_Btab =>
            Prev_Tab (State);

         --  F1-F7: Direct format selection
         when Key_F1 => State.Active_Tab := 0;
         when Key_F2 => State.Active_Tab := 1;
         when Key_F3 => State.Active_Tab := 2;
         when Key_F4 => State.Active_Tab := 3;
         when Key_F5 => State.Active_Tab := 4;
         when Key_F6 => State.Active_Tab := 5;
         when Key_F7 => State.Active_Tab := 6;

         --  Editor input
         when others =>
            if State.Focus = Editor then
               --  Handle basic text input
               if Key >= 32 and Key < 127 then
                  Append (State.Content, Character'Val (Key));
                  State.Modified := True;
               elsif Key = Key_Enter or Key = Character'Pos (ASCII.LF) then
                  Append (State.Content, ASCII.LF);
                  State.Modified := True;
               elsif Key = Key_Backspace and Length (State.Content) > 0 then
                  Delete (State.Content,
                          Length (State.Content),
                          Length (State.Content));
                  State.Modified := True;
               end if;
            end if;
      end case;
   end Handle_Key;

   procedure Load_File (State : in Out App_State; Path : String) is
      File    : Ada.Text_IO.File_Type;
      Line    : String (1 .. 4096);
      Last    : Natural;
   begin
      if not Ada.Directories.Exists (Path) then
         State.Status_Message := To_Unbounded_String ("File not found: " & Path);
         return;
      end if;

      Ada.Text_IO.Open (File, Ada.Text_IO.In_File, Path);
      State.Content := Null_Unbounded_String;

      while not Ada.Text_IO.End_Of_File (File) loop
         Ada.Text_IO.Get_Line (File, Line, Last);
         Append (State.Content, Line (1 .. Last));
         if not Ada.Text_IO.End_Of_File (File) then
            Append (State.Content, ASCII.LF);
         end if;
      end loop;

      Ada.Text_IO.Close (File);

      State.File_Path := To_Unbounded_String (Path);
      State.Modified := False;
      State.Status_Message := To_Unbounded_String ("Loaded: " & Path);

      --  Detect format from extension
      declare
         Ext : constant String := Ada.Directories.Extension (Path);
      begin
         if Ext = "txt" then
            State.Original_Format := Plain_Text;
            State.Active_Tab := 0;
         elsif Ext = "md" or Ext = "markdown" then
            State.Original_Format := Markdown;
            State.Active_Tab := 1;
         elsif Ext = "adoc" or Ext = "asciidoc" then
            State.Original_Format := AsciiDoc;
            State.Active_Tab := 2;
         elsif Ext = "dj" or Ext = "djot" then
            State.Original_Format := Djot;
            State.Active_Tab := 3;
         elsif Ext = "org" then
            State.Original_Format := Org_Mode;
            State.Active_Tab := 4;
         elsif Ext = "rst" then
            State.Original_Format := RST;
            State.Active_Tab := 5;
         elsif Ext = "typ" then
            State.Original_Format := Typst;
            State.Active_Tab := 6;
         end if;
      end;

   exception
      when others =>
         State.Status_Message := To_Unbounded_String ("Error loading: " & Path);
   end Load_File;

   procedure Save_File (State : in Out App_State) is
      File : Ada.Text_IO.File_Type;
      Path : constant String := To_String (State.File_Path);
   begin
      Ada.Text_IO.Create (File, Ada.Text_IO.Out_File, Path);
      Ada.Text_IO.Put (File, To_String (State.Content));
      Ada.Text_IO.Close (File);

      State.Modified := False;
      State.Status_Message := To_Unbounded_String ("Saved: " & Path);

   exception
      when others =>
         State.Status_Message := To_Unbounded_String ("Error saving: " & Path);
   end Save_File;

   procedure Next_Tab (State : in Out App_State) is
   begin
      State.Active_Tab := (State.Active_Tab + 1) mod 7;
   end Next_Tab;

   procedure Prev_Tab (State : in Out App_State) is
   begin
      if State.Active_Tab = 0 then
         State.Active_Tab := 6;
      else
         State.Active_Tab := State.Active_Tab - 1;
      end if;
   end Prev_Tab;

end Formatrix_TUI.App;
