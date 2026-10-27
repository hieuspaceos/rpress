<?php
	/**
	 * A widget that show recent event ( Specified by category ).
	 */

	add_action('widgets_init', 'gdlr_core_recent_event_widget');
	if( !function_exists('gdlr_core_recent_event_widget') ){
		function gdlr_core_recent_event_widget() {
			register_widget( 'Goodlayers_Core_Recent_Event_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Recent_Event_Widget') ){
		class Goodlayers_Core_Recent_Event_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-recent-event-widget', 
					esc_html__('Recent Event Widget ( Goodlayers )', 'chariti'), 
					array('description' => esc_html__('A widget that show latest events', 'chariti'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$category = empty($instance['category'])? '': $instance['category'];
				$num_fetch = empty($instance['num-fetch'])? '': $instance['num-fetch'];
				
				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}
					
				// Widget Content
				echo '<div class="gdlr-core-recent-event-widget gdlr-core-item-rvpdlr" >';
				echo gdlr_core_pb_element_event::get_content(array(
					'category' => $category,
					'num-fetch' => $num_fetch,
					'event-style' => 'widget'
				));
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
							'title' => esc_html__('Title', 'chariti'),
							'value' => (isset($instance['title'])? $instance['title']: '')
						),
						'category' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('category'),
							'name' => $this->get_field_name('category'),
							'title' => esc_html__('Category', 'chariti'),
							'options' => gdlr_core_get_term_list('tribe_events_cat', '', true),
							'value' => (isset($instance['category'])? $instance['category']: '')
						),
						'num-fetch' => array(
							'type' => 'text',
							'id' => $this->get_field_id('num-fetch'),
							'name' => $this->get_field_name('num-fetch'),
							'title' => esc_html__('Display Number', 'chariti'), 
							'value' => (isset($instance['num-fetch'])? $instance['num-fetch']: '3')
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