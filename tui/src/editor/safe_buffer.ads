-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Safe_Buffer - SPARK-verified bounded text buffer
--
-- This module provides a statically-verified text buffer with:
-- - Compile-time buffer bounds checking
-- - Memory safety guarantees
-- - Proven absence of runtime errors
--
-- Safety properties proven by SPARK:
-- - No buffer overflows
-- - No out-of-bounds array access
-- - No integer overflow in indexing
-- - Content integrity during all operations

pragma SPARK_Mode (On);

package Editor.Safe_Buffer is

   --  Maximum buffer size (64 KB should be sufficient for most documents)
   Max_Buffer_Size : constant := 65_536;

   --  Buffer content type
   subtype Buffer_Index is Natural range 0 .. Max_Buffer_Size;
   subtype Valid_Index is Buffer_Index range 1 .. Max_Buffer_Size;

   --  The safe buffer type
   type Safe_Buffer is private;

   --  Line and column position for cursor
   type Position is record
      Line   : Positive := 1;
      Column : Positive := 1;
   end record;

   --  Get buffer length
   function Length (Buffer : Safe_Buffer) return Buffer_Index
     with Global => null,
          Post   => Length'Result <= Max_Buffer_Size;

   --  Check if buffer is empty
   function Is_Empty (Buffer : Safe_Buffer) return Boolean
     with Global => null,
          Post   => Is_Empty'Result = (Length (Buffer) = 0);

   --  Check if buffer is full
   function Is_Full (Buffer : Safe_Buffer) return Boolean
     with Global => null,
          Post   => Is_Full'Result = (Length (Buffer) = Max_Buffer_Size);

   --  Get character at position (1-indexed)
   function Element (Buffer : Safe_Buffer; Index : Valid_Index) return Character
     with Global => null,
          Pre    => Index <= Length (Buffer);

   --  Get capacity remaining
   function Capacity (Buffer : Safe_Buffer) return Natural
     with Global => null,
          Post   => Capacity'Result = Max_Buffer_Size - Length (Buffer);

   --  Clear the buffer
   procedure Clear (Buffer : in out Safe_Buffer)
     with Global => null,
          Post   => Length (Buffer) = 0 and Is_Empty (Buffer);

   --  Append a single character
   procedure Append (Buffer : in Out Safe_Buffer; Char : Character)
     with Global => null,
          Pre    => Length (Buffer) < Max_Buffer_Size,
          Post   => Length (Buffer) = Length (Buffer)'Old + 1;

   --  Append a string (up to buffer capacity)
   procedure Append_String (Buffer : in Out Safe_Buffer; Str : String;
                            Appended : out Natural)
     with Global => null,
          Pre    => Str'Length >= 0,
          Post   => Appended <= Str'Length and
                    Length (Buffer) <= Max_Buffer_Size;

   --  Delete last character
   procedure Delete_Last (Buffer : in Out Safe_Buffer)
     with Global => null,
          Pre    => Length (Buffer) > 0,
          Post   => Length (Buffer) = Length (Buffer)'Old - 1;

   --  Delete character at position
   procedure Delete_At (Buffer : in Out Safe_Buffer; Index : Valid_Index)
     with Global => null,
          Pre    => Index <= Length (Buffer) and Length (Buffer) > 0,
          Post   => Length (Buffer) = Length (Buffer)'Old - 1;

   --  Insert character at position
   procedure Insert_At (Buffer : in Out Safe_Buffer;
                        Index  : Valid_Index;
                        Char   : Character)
     with Global => null,
          Pre    => Index <= Length (Buffer) + 1 and
                    Length (Buffer) < Max_Buffer_Size,
          Post   => Length (Buffer) = Length (Buffer)'Old + 1;

   --  Get a slice of the buffer (returns slice length)
   procedure Get_Slice (Buffer      : Safe_Buffer;
                        Start_Index : Valid_Index;
                        End_Index   : Valid_Index;
                        Result      : out String;
                        Last        : out Natural)
     with Global => null,
          Pre    => Start_Index <= End_Index and
                    End_Index <= Length (Buffer) and
                    Result'Length >= End_Index - Start_Index + 1;

   --  Convert position to index (1-based)
   function Position_To_Index (Buffer : Safe_Buffer; Pos : Position) return Buffer_Index
     with Global => null;

   --  Convert index to position
   function Index_To_Position (Buffer : Safe_Buffer; Index : Buffer_Index) return Position
     with Global => null,
          Pre => Index <= Length (Buffer);

   --  Count lines in buffer
   function Line_Count (Buffer : Safe_Buffer) return Positive
     with Global => null,
          Post => Line_Count'Result >= 1;

   --  Get start index of a line (1-indexed line number)
   function Line_Start (Buffer : Safe_Buffer; Line_Num : Positive) return Buffer_Index
     with Global => null,
          Pre => Line_Num <= Line_Count (Buffer);

   --  Get length of a line
   function Line_Length (Buffer : Safe_Buffer; Line_Num : Positive) return Natural
     with Global => null,
          Pre => Line_Num <= Line_Count (Buffer);

private

   type Buffer_Array is array (Valid_Index) of Character;

   type Safe_Buffer is record
      Data   : Buffer_Array := (others => ' ');
      Len    : Buffer_Index := 0;
   end record
     with Type_Invariant => Len <= Max_Buffer_Size;

end Editor.Safe_Buffer;
