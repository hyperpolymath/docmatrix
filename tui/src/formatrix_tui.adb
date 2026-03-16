-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Formatrix TUI - Terminal interface for Formatrix Docs
-- Main entry point

with Ada.Text_IO;
with Ada.Command_Line;
with Terminal_Interface.Curses;
with Formatrix_TUI.App;
with Formatrix_TUI.UI;

procedure Formatrix_TUI is
   use Terminal_Interface.Curses;

   --  Application state
   App_State : Formatrix_TUI.App.App_State;
begin
   --  Parse command line arguments
   if Ada.Command_Line.Argument_Count > 0 then
      Formatrix_TUI.App.Load_File
        (App_State, Ada.Command_Line.Argument (1));
   end if;

   --  Initialize curses
   Init_Screen;
   Set_Cbreak_Mode (SwitchOn => True);
   Set_Echo_Mode (SwitchOn => False);
   Set_KeyPad_Mode (Standard_Window, SwitchOn => True);

   --  Enable colors if supported
   if Has_Colors then
      Start_Color;
      Formatrix_TUI.UI.Setup_Colors;
   end if;

   --  Main event loop
   loop
      --  Render UI
      Formatrix_TUI.UI.Render (App_State);
      Refresh;

      --  Handle input
      declare
         Key : Key_Code := Get_Keystroke;
      begin
         Formatrix_TUI.App.Handle_Key (App_State, Key);

         exit when App_State.Should_Quit;
      end;
   end loop;

   --  Cleanup
   End_Windows;

exception
   when others =>
      End_Windows;
      Ada.Text_IO.Put_Line ("Error: Application terminated unexpectedly");
      raise;
end Formatrix_TUI;
