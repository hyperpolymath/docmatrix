-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Formatrix TUI - UI rendering

with Terminal_Interface.Curses;
with Formatrix_TUI.App;

package Formatrix_TUI.UI is

   use Terminal_Interface.Curses;

   --  Color pair constants
   Tab_Inactive    : constant Color_Pair := 1;
   Tab_Active      : constant Color_Pair := 2;
   Editor_Normal   : constant Color_Pair := 3;
   Status_Bar      : constant Color_Pair := 4;
   Dialog_Border   : constant Color_Pair := 5;

   --  Setup color pairs
   procedure Setup_Colors;

   --  Render the complete UI
   procedure Render (State : Formatrix_TUI.App.App_State);

   --  Render tab bar
   procedure Render_Tabs (State : Formatrix_TUI.App.App_State);

   --  Render editor area
   procedure Render_Editor (State : Formatrix_TUI.App.App_State);

   --  Render status bar
   procedure Render_Status (State : Formatrix_TUI.App.App_State);

   --  Render dialog overlay
   procedure Render_Dialog (State : Formatrix_TUI.App.App_State);

end Formatrix_TUI.UI;
