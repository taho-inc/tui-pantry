Feature: Top-Bar Tab Navigation
  As a widget developer
  I want top-level tabs organizing Widgets, Views, and Styles
  So that I can navigate between different pantry concerns

  Background:
    Given the pantry is running with ingredients across multiple tabs

  # ============================================================================
  # TAB DISPLAY
  # ============================================================================

  @core @local @native @navigation
  Scenario: Top bar displays all discovered tabs
    Then the top bar should display "tui-pantry" as the app name
    And the top bar should display tabs "Widgets", "Views", "Styles"
    And the "Widgets" tab should be active by default

  @core @local @native @rendering
  Scenario: Active tab is visually distinguished
    Given the "Widgets" tab is active
    Then the "Widgets" tab should use the accent color
    And the "Views" tab should use the dim text color
    And the "Styles" tab should use the dim text color

  @core @local @native @rendering
  Scenario: Top bar occupies one row above the main content
    Then the top bar should occupy exactly 1 row
    And the sidebar and preview should appear below the top bar

  # ============================================================================
  # TAB SWITCHING
  # ============================================================================

  @core @local @native @keyboard
  Scenario: Switch tabs with number keys
    Given the sidebar is focused
    When the user presses "2"
    Then the "Views" tab should become active
    When the user presses "3"
    Then the "Styles" tab should become active
    When the user presses "1"
    Then the "Widgets" tab should become active

  @core @local @native @keyboard
  Scenario: Cycle tabs forward with Tab key
    Given the "Widgets" tab is active
    When the user presses Tab
    Then the "Views" tab should become active
    When the user presses Tab
    Then the "Styles" tab should become active
    When the user presses Tab
    Then the "Widgets" tab should become active

  @core @local @native @keyboard
  Scenario: Cycle tabs backward with Shift-Tab
    Given the "Widgets" tab is active
    When the user presses Shift-Tab
    Then the "Styles" tab should become active
    When the user presses Shift-Tab
    Then the "Views" tab should become active

  @core @local @native @keyboard
  Scenario: Tab switching only works from sidebar focus
    Given the preview pane is focused
    When the user presses "2"
    Then the active tab should not change
    And the key should be forwarded to the ingredient

  # ============================================================================
  # SIDEBAR CONTENT PER TAB
  # ============================================================================

  @core @local @native @navigation
  Scenario: Widgets tab shows widget ingredient groups
    Given the "Widgets" tab is active
    Then the sidebar should display widget groups
    And each group should be expandable to show variants

  @core @local @native @navigation
  Scenario: Views tab shows view composition groups
    Given the "Views" tab is active
    Then the sidebar should display view groups
    And selecting a view should render the composed layout

  @core @local @native @navigation
  Scenario: Styles tab shows color and typography groups
    Given the "Styles" tab is active
    Then the sidebar should display "Colors" group
    And the sidebar should display "Typography" group
    And selecting "Brand Palette" should render color swatches

  # ============================================================================
  # STATE PRESERVATION
  # ============================================================================

  @core @local @native @navigation
  Scenario: Each tab preserves its own selection state
    Given the "Widgets" tab is active
    And the user selects "Node Table > Default"
    When the user switches to the "Styles" tab
    And the user selects "Colors > Brand Palette"
    And the user switches back to the "Widgets" tab
    Then "Node Table > Default" should still be selected

  @core @local @native @navigation
  Scenario: Each tab preserves its own expansion state
    Given the "Widgets" tab is active
    And the user collapses "Node Card" group
    When the user switches to the "Styles" tab
    And the user switches back to the "Widgets" tab
    Then the "Node Card" group should still be collapsed

  # ============================================================================
  # KEY HINTS
  # ============================================================================

  @core @local @native @rendering
  Scenario: Bottom bar shows tab switching hints
    Given the sidebar is focused
    Then the bottom bar should include "1-3" or "Tab" in the key hints

  @core @local @native @rendering
  Scenario: Preview focus hides tab hints
    Given the preview pane is focused
    Then the bottom bar should not include tab switching hints
