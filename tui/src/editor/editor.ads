-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Editor - Root package for editor components
--
-- This package hierarchy contains SPARK-verified components for
-- safety-critical text editing operations.

pragma SPARK_Mode (On);

package Editor is

   --  Configuration constants for editor
   Tab_Width        : constant := 4;
   Max_Line_Length  : constant := 1024;
   Max_Undo_Depth   : constant := 100;

end Editor;
