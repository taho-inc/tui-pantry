Feature: Sidebar Navigation Tree
  As a widget developer
  I want to browse ingredients organized by group in a sidebar
  So that I can find and select widget variants for preview

  Background:
    Given the harness is loaded with ingredients from multiple groups

  # ============================================================================
  # INITIAL STATE
  # ============================================================================

  @core @local @native @navigation
  Scenario: All groups are expanded on launch
    Then every group in the sidebar should be expanded
    And each group's variants should be visible beneath it

  @core @local @native @navigation
  Scenario: First variant is selected on launch
    Then the cursor should be on the first variant of the first group
    And the preview pane should display that variant

  @core @local @native @navigation
  Scenario: Sidebar header shows ingredient count
    Then the sidebar header should display "WIDGETS" followed by the total ingredient count

  # ============================================================================
  # GROUPING
  # ============================================================================

  @core @local @native @navigation
  Scenario: Ingredients are grouped by their group name
    Then the sidebar should display one group header per unique group value
    And each group header should list its variants beneath it
    And groups should appear in the order their first ingredient was registered

  @core @local @native @navigation
  Scenario: Group headers display expand/collapse caret
    Given a group is expanded
    Then its header should display a "down-pointing" caret
    Given a group is collapsed
    Then its header should display a "right-pointing" caret

  @core @local @native @navigation
  Scenario: Variants display diamond marker
    Then each variant entry should display a diamond marker followed by the ingredient name

  # ============================================================================
  # CURSOR MOVEMENT
  # ============================================================================

  @core @local @native @navigation
  Scenario: Cursor moves down through visible entries
    Given the cursor is not on the last visible entry
    When the user moves the cursor down
    Then the cursor should advance to the next visible entry

  @core @local @native @navigation
  Scenario: Cursor moves up through visible entries
    Given the cursor is not on the first visible entry
    When the user moves the cursor up
    Then the cursor should retreat to the previous visible entry

  @core @local @native @navigation
  Scenario: Cursor stops at the bottom boundary
    Given the cursor is on the last visible entry
    When the user moves the cursor down
    Then the cursor should remain on the last visible entry

  @core @local @native @navigation
  Scenario: Cursor stops at the top boundary
    Given the cursor is on the first visible entry
    When the user moves the cursor up
    Then the cursor should remain on the first visible entry

  # ============================================================================
  # EXPAND / COLLAPSE
  # ============================================================================

  @core @local @native @navigation
  Scenario: Collapse a group from its header
    Given the cursor is on an expanded group header
    When the user collapses the current entry
    Then the group's variants should be hidden
    And the group header should display a "right-pointing" caret

  @core @local @native @navigation
  Scenario: Expand a collapsed group from its header
    Given the cursor is on a collapsed group header
    When the user expands the current entry
    Then the group's variants should become visible
    And the group header should display a "down-pointing" caret

  @core @local @native @navigation
  Scenario: Toggle group via Enter on a group header
    Given the cursor is on an expanded group header
    When the user presses Enter
    Then the group should collapse
    When the user presses Enter again
    Then the group should expand

  @core @local @native @navigation
  Scenario: Collapse from a variant moves cursor to parent group
    Given the cursor is on a variant within an expanded group
    When the user collapses the current entry
    Then the parent group should collapse
    And the cursor should move to the parent group header

  @core @local @native @navigation
  Scenario: Collapsing a group hides its variants from the visible list
    Given a group has 3 visible variants
    When the user collapses that group
    Then the total visible entry count should decrease by 3

  # ============================================================================
  # SELECTION
  # ============================================================================

  @core @local @native @navigation
  Scenario: Cursor on a variant selects it for preview
    Given the cursor is on a variant entry
    Then the preview pane should render that ingredient

  @core @local @native @navigation
  Scenario: Cursor on a group header shows no preview
    Given the cursor is on a group header
    Then the preview pane should display a prompt to select an ingredient

  # ============================================================================
  # FOCUS MODE
  # ============================================================================

  @core @local @native @navigation
  Scenario: Focus starts on sidebar
    Then the focus should be on the sidebar
    And the preview pane border should be gray

  @core @local @native @navigation
  Scenario: Enter on an interactive variant focuses the preview pane
    Given the cursor is on an interactive variant
    When the user presses Enter
    Then the focus should move to the preview pane
    And the preview pane border should turn orange
    And the bottom bar hints should show "navigate" and "back"

  @core @local @native @navigation
  Scenario: Enter on a non-interactive variant does not focus the preview pane
    Given the cursor is on a non-interactive variant
    When the user presses Enter
    Then the focus should remain on the sidebar

  @core @local @native @navigation
  Scenario: Esc exits preview focus and returns to sidebar
    Given the preview pane is focused
    When the user presses Esc
    Then the focus should return to the sidebar
    And the preview pane border should return to gray
    And the bottom bar hints should show sidebar controls

  @core @local @native @navigation
  Scenario: Keys forward to the ingredient while preview is focused
    Given the preview pane is focused on an interactive ingredient
    When the user presses a navigation key
    Then the ingredient's handle_key method should receive the key
    And the sidebar cursor should not move

  @core @local @native @navigation
  Scenario: Sidebar navigation keys are blocked while preview is focused
    Given the preview pane is focused
    When the user presses "j"
    Then the sidebar cursor should not move
    And the key should be forwarded to the ingredient
