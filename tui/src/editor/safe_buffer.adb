-- SPDX-License-Identifier: PMPL-1.0-or-later
--
-- Safe_Buffer - SPARK-verified bounded text buffer (body)

pragma SPARK_Mode (On);

package body Editor.Safe_Buffer is

   ---------------
   -- Length --
   ---------------

   function Length (Buffer : Safe_Buffer) return Buffer_Index is
   begin
      return Buffer.Len;
   end Length;

   --------------
   -- Is_Empty --
   --------------

   function Is_Empty (Buffer : Safe_Buffer) return Boolean is
   begin
      return Buffer.Len = 0;
   end Is_Empty;

   -------------
   -- Is_Full --
   -------------

   function Is_Full (Buffer : Safe_Buffer) return Boolean is
   begin
      return Buffer.Len = Max_Buffer_Size;
   end Is_Full;

   -------------
   -- Element --
   -------------

   function Element (Buffer : Safe_Buffer; Index : Valid_Index) return Character is
   begin
      return Buffer.Data (Index);
   end Element;

   --------------
   -- Capacity --
   --------------

   function Capacity (Buffer : Safe_Buffer) return Natural is
   begin
      return Max_Buffer_Size - Buffer.Len;
   end Capacity;

   -----------
   -- Clear --
   -----------

   procedure Clear (Buffer : in Out Safe_Buffer) is
   begin
      Buffer.Len := 0;
      --  Note: We don't need to clear Data, just reset length
   end Clear;

   ------------
   -- Append --
   ------------

   procedure Append (Buffer : in Out Safe_Buffer; Char : Character) is
   begin
      Buffer.Len := Buffer.Len + 1;
      Buffer.Data (Buffer.Len) := Char;
   end Append;

   -------------------
   -- Append_String --
   -------------------

   procedure Append_String (Buffer : in Out Safe_Buffer; Str : String;
                            Appended : out Natural) is
      Space_Left  : constant Natural := Max_Buffer_Size - Buffer.Len;
      To_Append   : Natural;
   begin
      --  Calculate how many characters we can actually append
      if Str'Length <= Space_Left then
         To_Append := Str'Length;
      else
         To_Append := Space_Left;
      end if;

      --  Append characters one by one (SPARK-friendly loop)
      for I in 0 .. To_Append - 1 loop
         pragma Loop_Invariant (Buffer.Len = Buffer.Len'Loop_Entry + I);
         pragma Loop_Invariant (Buffer.Len < Max_Buffer_Size);
         Buffer.Len := Buffer.Len + 1;
         Buffer.Data (Buffer.Len) := Str (Str'First + I);
      end loop;

      Appended := To_Append;
   end Append_String;

   -----------------
   -- Delete_Last --
   -----------------

   procedure Delete_Last (Buffer : in Out Safe_Buffer) is
   begin
      Buffer.Len := Buffer.Len - 1;
   end Delete_Last;

   ---------------
   -- Delete_At --
   ---------------

   procedure Delete_At (Buffer : in Out Safe_Buffer; Index : Valid_Index) is
   begin
      --  Shift all characters after Index left by one
      for I in Index .. Buffer.Len - 1 loop
         pragma Loop_Invariant (Buffer.Len = Buffer.Len'Loop_Entry);
         Buffer.Data (I) := Buffer.Data (I + 1);
      end loop;
      Buffer.Len := Buffer.Len - 1;
   end Delete_At;

   ---------------
   -- Insert_At --
   ---------------

   procedure Insert_At (Buffer : in Out Safe_Buffer;
                        Index  : Valid_Index;
                        Char   : Character) is
   begin
      --  Shift all characters from Index right by one
      for I in reverse Index .. Buffer.Len loop
         pragma Loop_Invariant (Buffer.Len = Buffer.Len'Loop_Entry);
         Buffer.Data (I + 1) := Buffer.Data (I);
      end loop;
      Buffer.Data (Index) := Char;
      Buffer.Len := Buffer.Len + 1;
   end Insert_At;

   ---------------
   -- Get_Slice --
   ---------------

   procedure Get_Slice (Buffer      : Safe_Buffer;
                        Start_Index : Valid_Index;
                        End_Index   : Valid_Index;
                        Result      : out String;
                        Last        : out Natural) is
      Slice_Len : constant Natural := End_Index - Start_Index + 1;
   begin
      for I in 0 .. Slice_Len - 1 loop
         pragma Loop_Invariant (I < Slice_Len);
         Result (Result'First + I) := Buffer.Data (Start_Index + I);
      end loop;
      Last := Result'First + Slice_Len - 1;
   end Get_Slice;

   -----------------------
   -- Position_To_Index --
   -----------------------

   function Position_To_Index (Buffer : Safe_Buffer; Pos : Position) return Buffer_Index is
      Current_Line : Positive := 1;
      Index        : Buffer_Index := 0;
   begin
      if Buffer.Len = 0 then
         return 0;
      end if;

      --  Find the start of the requested line
      for I in 1 .. Buffer.Len loop
         pragma Loop_Invariant (Current_Line <= Pos.Line);
         pragma Loop_Invariant (I <= Buffer.Len);

         if Current_Line = Pos.Line then
            --  We're on the right line, calculate column
            Index := I + Pos.Column - 2;  -- -2 because both are 1-indexed
            if Index > Buffer.Len then
               Index := Buffer.Len;
            end if;
            return Index;
         end if;

         if Buffer.Data (I) = ASCII.LF then
            Current_Line := Current_Line + 1;
         end if;
      end loop;

      --  If we didn't find the line, return end of buffer
      return Buffer.Len;
   end Position_To_Index;

   -----------------------
   -- Index_To_Position --
   -----------------------

   function Index_To_Position (Buffer : Safe_Buffer; Index : Buffer_Index) return Position is
      Result : Position := (Line => 1, Column => 1);
   begin
      if Index = 0 or Buffer.Len = 0 then
         return Result;
      end if;

      for I in 1 .. Index loop
         pragma Loop_Invariant (I <= Index);
         pragma Loop_Invariant (Result.Line >= 1);
         pragma Loop_Invariant (Result.Column >= 1);

         if I > 1 and then Buffer.Data (I - 1) = ASCII.LF then
            Result.Line := Result.Line + 1;
            Result.Column := 1;
         else
            Result.Column := Result.Column + 1;
         end if;
      end loop;

      return Result;
   end Index_To_Position;

   ----------------
   -- Line_Count --
   ----------------

   function Line_Count (Buffer : Safe_Buffer) return Positive is
      Count : Positive := 1;
   begin
      for I in 1 .. Buffer.Len loop
         pragma Loop_Invariant (Count >= 1);
         pragma Loop_Invariant (I <= Buffer.Len);
         if Buffer.Data (I) = ASCII.LF then
            Count := Count + 1;
         end if;
      end loop;
      return Count;
   end Line_Count;

   ----------------
   -- Line_Start --
   ----------------

   function Line_Start (Buffer : Safe_Buffer; Line_Num : Positive) return Buffer_Index is
      Current_Line : Positive := 1;
   begin
      if Line_Num = 1 then
         return (if Buffer.Len > 0 then 1 else 0);
      end if;

      for I in 1 .. Buffer.Len loop
         pragma Loop_Invariant (Current_Line < Line_Num);
         pragma Loop_Invariant (I <= Buffer.Len);

         if Buffer.Data (I) = ASCII.LF then
            Current_Line := Current_Line + 1;
            if Current_Line = Line_Num then
               return (if I < Buffer.Len then I + 1 else Buffer.Len);
            end if;
         end if;
      end loop;

      return Buffer.Len;
   end Line_Start;

   -----------------
   -- Line_Length --
   -----------------

   function Line_Length (Buffer : Safe_Buffer; Line_Num : Positive) return Natural is
      Start_Idx : constant Buffer_Index := Line_Start (Buffer, Line_Num);
      Len       : Natural := 0;
   begin
      if Start_Idx = 0 then
         return 0;
      end if;

      for I in Start_Idx .. Buffer.Len loop
         pragma Loop_Invariant (I <= Buffer.Len);
         pragma Loop_Invariant (Len <= Buffer.Len);

         if Buffer.Data (I) = ASCII.LF then
            return Len;
         end if;
         Len := Len + 1;
      end loop;

      return Len;
   end Line_Length;

end Editor.Safe_Buffer;
