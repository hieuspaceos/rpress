<?php
	/**
	 * A widget that show recent posts ( Specified by category ).
	 */

	add_action('widgets_init', 'gdlr_core_instagram_widget');
	if( !function_exists('gdlr_core_instagram_widget') ){
		function gdlr_core_instagram_widget() {
			register_widget( 'Goodlayers_Core_Instagram_Widget' );
		}
	}

	if( !class_exists('Goodlayers_Core_Instagram_Widget') ){
		class Goodlayers_Core_Instagram_Widget extends WP_Widget{

			// Initialize the widget
			function __construct() {

				parent::__construct(
					'gdlr-core-instagram-widget', 
					esc_html__('Instagram Widget ( Goodlayers )', 'goodlayers-core'), 
					array('description' => esc_html__('A widget that show instagram image', 'goodlayers-core'))
				);  

			}

			// Output of the widget
			function widget( $args, $instance ) {
	
				$title = empty($instance['title'])? '': apply_filters('widget_title', $instance['title']);
				$username = empty($instance['username'])? '': $instance['username'];
				$client_id = empty($instance['client-id'])? '': $instance['client-id'];
				$num_fetch = empty($instance['num-fetch'])? '6': $instance['num-fetch'];
				$column_size = empty($instance['column'])? '20': (60 / intval($instance['column']));
				$cache_time = empty($instance['cache-time'])? '': $instance['cache-time'];
				
				// Opening of widget
				echo gdlr_core_escape_content($args['before_widget']);
				
				// Open of title tag
				if( !empty($title) ){ 
					echo gdlr_core_escape_content($args['before_title'] . $title . $args['after_title']); 
				}
					
				// Widget Content
				$instagram_images = gdlr_core_pb_element_instagram::get_instagram($username, $client_id, $num_fetch, $cache_time);
				 
				if( is_wp_error($instagram_images) ){
					echo $instagram_images->get_error_message();
				}else{
					if( !empty($instagram_images) ){
						$column_count = 0;
						$lightbox_group = gdlr_core_image_group_id();
						
						echo '<div class="gdlr-core-instagram-widget clearfix" >';
						foreach( $instagram_images as $image ){

							if( !empty($image['thumbnail']['url']) ){
								
								$full_image = empty($image['large']['url'])? $image['thumbnail']['url']: $image['large']['url'];
									
								$column_class  = ' gdlr-core-column-' . $column_size;
								$column_class .= ($column_count % 60 == 0)? ' gdlr-core-column-first': '';
								
								echo '<div class="' . esc_attr($column_class) . ' gdlr-core-media-image" >';
								echo '<a ' . gdlr_core_get_lightbox_atts(array(
									'url'=>$full_image, 
									'group' => $lightbox_group,
									'caption' => (empty($image['description'])? '': esc_attr($image['description']))
								)) . ' >';
								echo '<img src="' . esc_url($full_image) . '" ';
								echo empty($image['thumbnail']['width'])? '': ' width="' . esc_attr($image['thumbnail']['width']) . '" ';
								echo empty($image['thumbnail']['height'])? '': ' height="' . esc_attr($image['thumbnail']['height']) . '" ';
								echo ' alt="' . (empty($image['description'])? '': esc_attr($image['description'])) . '" ';
								echo ' />';
								echo '</a>';
								echo '</div>';

								$column_count += $column_size;

							}
						}
						echo '</div>';
					} 
				}
				
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
						'username' => array(
							'type' => 'text',
							'id' => $this->get_field_id('username'),
							'name' => $this->get_field_name('username'),
							'title' => esc_html__('User Name', 'goodlayers-core'),
							'value' => (isset($instance['username'])? $instance['username']: '')
						),
						'client-id' => array(
							'type' => 'text',
							'id' => $this->get_field_id('client-id'),
							'name' => $this->get_field_name('client-id'),
							'title' => esc_html__('Access Token', 'goodlayers-core'),
							'value' => (isset($instance['client-id'])? $instance['client-id']: '')
						),
						'num-fetch' => array(
							'type' => 'text',
							'id' => $this->get_field_id('num-fetch'),
							'name' => $this->get_field_name('num-fetch'),
							'title' => esc_html__('Num Fetch', 'goodlayers-core'), 
							'options' => gdlr_core_get_thumbnail_list(),
							'value' => (isset($instance['num-fetch'])? $instance['num-fetch']: '6')
						),
						'column' => array(
							'type' => 'combobox',
							'id' => $this->get_field_id('column'),
							'name' => $this->get_field_name('column'),
							'title' => esc_html__('Column', 'goodlayers-core'),
							'options' => array( 1=>1, 2=>2, 3=>3, 4=>4, 5=>5, 6=>6),
							'value' => (isset($instance['column'])? $instance['column']: '3')
						),
						'cache-time' => array(
							'type' => 'text',
							'id' => $this->get_field_id('cache-time'),
							'name' => $this->get_field_name('cache-time'),
							'title' => esc_html__('Cache Time', 'goodlayers-core'),
							'value' => (isset($instance['cache-time'])? $instance['cache-time']: '1')
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