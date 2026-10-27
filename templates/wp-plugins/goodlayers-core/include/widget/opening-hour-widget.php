<?php
	/**
	 * A widget that show recent posts ( Specified by category ).
	 */

	add_action('widgets_init', 'gdlr_core_opening_hour_widget');
	if( !function_exists('gdlr_core_opening_hour_widget') ){
		function gdlr_core_opening_hour_widget() {
			register_widget( 'Goodlayers_Core_Opening_Hour_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Opening_Hour_Widget') ){
		class Goodlayers_Core_Opening_Hour_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-opening-hour-widget', 
					esc_html__('Opening Hour Widget ( Goodlayers )', 'goodlayers-core'), 
					array('description' => esc_html__('A widget that show opening hours', 'goodlayers-core'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$color = empty($instance['color'])? '': $instance['color'];
				$icon = empty($instance['icon'])? 'fa fa-clock-o': $instance['icon'];
					
				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}
					
				// Widget Content
				echo '<div class="gdlr-core-opening-hour-widget" ' . gdlr_core_esc_style(array(
					'color' => $color
				)) . ' >';
				for($i = 1; $i <= 7; $i++){
					$day_text = empty($instance['day' . $i . '-text'])? '': $instance['day' . $i . '-text'];
					$day_time = empty($instance['day' . $i . '-time'])? '': $instance['day' . $i . '-time'];

					if( !empty($day_text) || !empty($day_time) ){
						echo '<div class="gdlr-core-opening-hour-widget-list clearfix" >';
						echo '<span class="gdlr-core-head" ><i class="' . esc_attr($icon) . '" ></i>' . gdlr_core_text_filter($day_text) . '</span>';
						echo '<span class="gdlr-core-tail" >' . gdlr_core_text_filter($day_time) . '</span>';
						echo '</div>';
					}
				}
				echo '</div>';
				
				
				// Closing of widget
				echo gdlr_core_escape_content($args['after_widget']);

			}

			// Widget Form
			function form( $instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					gdlr_core_widget_util::get_option(array(
						'title' => array(
							'type' => 'text',
							'id' => $this->get_field_id('title'),
							'name' => $this->get_field_name('title'),
							'title' => esc_html__('Title', 'goodlayers-core'),
							'value' => (isset($instance['title'])? $instance['title']: '')
						),
						'color' => array(
							'type' => 'colorpicker',
							'id' => $this->get_field_id('color'),
							'name' => $this->get_field_name('color'),
							'title' => esc_html__('Text Color', 'goodlayers-core'),
							'value' => (isset($instance['color'])? $instance['color']: '')
						),
						'icon' => array(
							'type' => 'text',
							'id' => $this->get_field_id('icon'),
							'name' => $this->get_field_name('icon'),
							'title' => esc_html__('Icon Class', 'goodlayers-core'),
							'value' => (isset($instance['icon'])? $instance['icon']: '')
						),
						'day1-text' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day1-text'),
							'name' => $this->get_field_name('day1-text'),
							'title' => esc_html__('Day1 Text', 'goodlayers-core'),
							'value' => (isset($instance['day1-text'])? $instance['day1-text']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day1-time' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day1-time'),
							'name' => $this->get_field_name('day1-time'),
							'title' => esc_html__('Day1 Time', 'goodlayers-core'),
							'value' => (isset($instance['day1-time'])? $instance['day1-time']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day2-text' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day2-text'),
							'name' => $this->get_field_name('day2-text'),
							'title' => esc_html__('Day2 Text', 'goodlayers-core'),
							'value' => (isset($instance['day2-text'])? $instance['day2-text']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day2-time' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day2-time'),
							'name' => $this->get_field_name('day2-time'),
							'title' => esc_html__('Day2 Time', 'goodlayers-core'),
							'value' => (isset($instance['day2-time'])? $instance['day2-time']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day3-text' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day3-text'),
							'name' => $this->get_field_name('day3-text'),
							'title' => esc_html__('Day3 Text', 'goodlayers-core'),
							'value' => (isset($instance['day3-text'])? $instance['day3-text']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day3-time' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day3-time'),
							'name' => $this->get_field_name('day3-time'),
							'title' => esc_html__('Day3 Time', 'goodlayers-core'),
							'value' => (isset($instance['day3-time'])? $instance['day3-time']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day4-text' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day4-text'),
							'name' => $this->get_field_name('day4-text'),
							'title' => esc_html__('Day4 Text', 'goodlayers-core'),
							'value' => (isset($instance['day4-text'])? $instance['day4-text']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day4-time' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day4-time'),
							'name' => $this->get_field_name('day4-time'),
							'title' => esc_html__('Day4 Time', 'goodlayers-core'),
							'value' => (isset($instance['day4-time'])? $instance['day4-time']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day5-text' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day5-text'),
							'name' => $this->get_field_name('day5-text'),
							'title' => esc_html__('Day5 Text', 'goodlayers-core'),
							'value' => (isset($instance['day5-text'])? $instance['day5-text']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day5-time' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day5-time'),
							'name' => $this->get_field_name('day5-time'),
							'title' => esc_html__('Day5 Time', 'goodlayers-core'),
							'value' => (isset($instance['day5-time'])? $instance['day5-time']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day6-text' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day6-text'),
							'name' => $this->get_field_name('day6-text'),
							'title' => esc_html__('Day6 Text', 'goodlayers-core'),
							'value' => (isset($instance['day6-text'])? $instance['day6-text']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day6-time' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day6-time'),
							'name' => $this->get_field_name('day6-time'),
							'title' => esc_html__('Day6 Time', 'goodlayers-core'),
							'value' => (isset($instance['day6-time'])? $instance['day6-time']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day7-text' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day7-text'),
							'name' => $this->get_field_name('day7-text'),
							'title' => esc_html__('Day7 Text', 'goodlayers-core'),
							'value' => (isset($instance['day7-text'])? $instance['day7-text']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
						'day7-time' => array(
							'type' => 'text',
							'id' => $this->get_field_id('day7-time'),
							'name' => $this->get_field_name('day7-time'),
							'title' => esc_html__('Day7 Time', 'goodlayers-core'),
							'value' => (isset($instance['day7-time'])? $instance['day7-time']: ''),
							'style' => array('half', 'margin-bottom-0')
						),
					));
				}

			}
			
			// Update the widget
			function update( $new_instance, $old_instance ) {

				if( class_exists('gdlr_core_widget_util') ){
					return gdlr_core_widget_util::get_option_update($new_instance);
				}

				return $new_instance;
			}	
		} // class
	} // class_exists
?>